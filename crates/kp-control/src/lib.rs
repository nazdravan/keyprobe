//! L6: control channel — newline-delimited JSON over localhost TCP, mapped onto engine commands.
//!
//! Lets agents and scripts drive the running app exactly like a user (Rule 4):
//!
//! ```text
//! → {"id": 1, "cmd": "key.down", "params": {"code": "KeyA"}}
//! ← {"id": 1, "ok": true, "result": null}
//! → {"id": 2, "cmd": "ui.screenshot", "params": {"path": "/tmp/shot.png"}}
//! ← {"id": 2, "ok": true, "result": {"path": "/tmp/shot.png", "width": 960, "height": 420}}
//! ```
//!
//! Besides every registered command, `cmd` may be `ping`, `commands` (the registry), `state` (the
//! document as JSON) or `ui.screenshot` (handled by the UI, which owns the pixels).
//!
//! The server binds 127.0.0.1 only and runs only when the app is started with `--control PORT`.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use kp_engine::App;
use serde::Deserialize;
use serde_json::{Value, json};

/// Longest accepted request line, in bytes.
pub const MAX_LINE: usize = 64 * 1024;
/// How long a connection waits for the UI thread to answer one request.
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Echoed back in the reply; any JSON value.
    #[serde(default)]
    pub id: Value,
    pub cmd: String,
    #[serde(default)]
    pub params: Value,
}

/// What the UI thread must do with a request.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Answer right away.
    Reply(Value),
    /// Capture the next frame to this PNG path, then answer.
    Screenshot { path: PathBuf },
}

pub fn parse(line: &str) -> Result<Request, String> {
    serde_json::from_str(line).map_err(|e| format!("bad request: {e}"))
}

/// Runs a request against the engine. Screenshots are returned as an [`Action`] because only the
/// UI can read pixels.
pub fn handle(app: &mut App, req: &Request) -> Result<Action, String> {
    match req.cmd.as_str() {
        "ping" => Ok(Action::Reply(json!("pong"))),
        "commands" => Ok(Action::Reply(Value::Array(
            app.registry()
                .specs()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "label": s.label,
                        "menu": s.menu,
                        "params": s.params,
                        "enabled": app.enabled(s.id).is_ok(),
                    })
                })
                .collect(),
        ))),
        "state" => serde_json::to_value(app.session().doc())
            .map(Action::Reply)
            .map_err(|e| e.to_string()),
        "ui.screenshot" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct P {
                path: PathBuf,
            }
            let P { path } = serde_json::from_value(req.params.clone())
                .map_err(|e| format!("ui.screenshot: bad params: {e}"))?;
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                return Err("ui.screenshot: path must end in .png".into());
            }
            Ok(Action::Screenshot { path })
        }
        id => app
            .run(id, &req.params)
            .map(Action::Reply)
            .map_err(|e| e.to_string()),
    }
}

/// One reply line (no trailing newline).
pub fn reply(id: &Value, result: Result<Value, String>) -> String {
    match result {
        Ok(v) => json!({"id": id, "ok": true, "result": v}),
        Err(e) => json!({"id": id, "ok": false, "error": e}),
    }
    .to_string()
}

/// A request waiting for the UI thread, with the way back to its connection.
#[derive(Debug)]
pub struct Pending {
    pub request: Request,
    pub respond: Sender<String>,
}

impl Pending {
    pub fn answer(&self, result: Result<Value, String>) {
        // The connection may have gone away; nothing to do then.
        let _ = self.respond.send(reply(&self.request.id, result));
    }
}

/// Starts the server on 127.0.0.1:`port` (0 = any free port). Requests arrive on the returned
/// receiver; `wake` is called after each one so the UI thread polls even when idle.
pub fn serve(
    port: u16,
    wake: Arc<dyn Fn() + Send + Sync>,
) -> io::Result<(SocketAddr, Receiver<Pending>)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    let addr = listener.local_addr()?;
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("control-accept".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let tx = tx.clone();
                let wake = Arc::clone(&wake);
                let _ = std::thread::Builder::new()
                    .name("control-conn".into())
                    .spawn(move || connection(stream, &tx, wake.as_ref()));
            }
        })?;
    Ok((addr, rx))
}

fn connection(stream: TcpStream, tx: &Sender<Pending>, wake: &(dyn Fn() + Send + Sync)) {
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    loop {
        let line = match read_line(&mut reader) {
            Ok(Some(line)) => line,
            Ok(None) => return,
            Err(e) => {
                let _ = writeln!(out, "{}", reply(&Value::Null, Err(e)));
                return;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let answer = match parse(&line) {
            Err(e) => reply(&Value::Null, Err(e)),
            Ok(request) => {
                let id = request.id.clone();
                let (respond, back) = mpsc::channel();
                if tx.send(Pending { request, respond }).is_err() {
                    return; // UI is gone.
                }
                wake();
                back.recv_timeout(REPLY_TIMEOUT)
                    .unwrap_or_else(|_| reply(&id, Err("timed out waiting for the UI".into())))
            }
        };
        if writeln!(out, "{answer}").is_err() {
            return;
        }
    }
}

/// Reads one line of at most [`MAX_LINE`] bytes; `Ok(None)` at end of stream.
fn read_line(reader: &mut impl BufRead) -> Result<Option<String>, String> {
    let mut buf = Vec::new();
    let n = reader
        .take(MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut buf)
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(None);
    }
    if buf.len() > MAX_LINE {
        return Err(format!("request line longer than {MAX_LINE} bytes"));
    }
    String::from_utf8(buf)
        .map(Some)
        .map_err(|_| "request is not UTF-8".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    fn run(app: &mut App, line: &str) -> Result<Action, String> {
        handle(app, &parse(line)?)
    }

    #[test]
    fn commands_state_and_engine_calls() {
        let mut app = App::default();
        assert_eq!(
            run(&mut app, r#"{"cmd":"ping"}"#),
            Ok(Action::Reply(json!("pong")))
        );
        run(
            &mut app,
            r#"{"id":1,"cmd":"key.down","params":{"code":"KeyA"}}"#,
        )
        .unwrap();
        let Ok(Action::Reply(state)) = run(&mut app, r#"{"cmd":"state"}"#) else {
            panic!("state failed");
        };
        assert_eq!(state["test"]["presses"]["KeyA"], 1);
        let Ok(Action::Reply(list)) = run(&mut app, r#"{"cmd":"commands"}"#) else {
            panic!("commands failed");
        };
        assert!(
            list.as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == "test.reset" && c["enabled"] == true)
        );
        assert!(
            run(&mut app, r#"{"cmd":"nope"}"#)
                .unwrap_err()
                .contains("unknown command")
        );
    }

    #[test]
    fn screenshot_params() {
        let mut app = App::default();
        assert_eq!(
            run(
                &mut app,
                r#"{"cmd":"ui.screenshot","params":{"path":"/tmp/a.png"}}"#
            ),
            Ok(Action::Screenshot {
                path: "/tmp/a.png".into()
            })
        );
        assert!(
            run(
                &mut app,
                r#"{"cmd":"ui.screenshot","params":{"path":"/tmp/a.txt"}}"#
            )
            .is_err()
        );
        assert!(run(&mut app, r#"{"cmd":"ui.screenshot"}"#).is_err());
    }

    #[test]
    fn reply_shapes() {
        assert_eq!(
            reply(&json!(7), Ok(json!(null))),
            r#"{"id":7,"ok":true,"result":null}"#
        );
        assert_eq!(
            reply(&json!("a"), Err("x".into())),
            r#"{"error":"x","id":"a","ok":false}"#
        );
    }

    #[test]
    fn line_limits() {
        let mut ok = Cursor::new(b"{\"cmd\":\"ping\"}\nrest".to_vec());
        assert_eq!(read_line(&mut ok).unwrap().unwrap(), "{\"cmd\":\"ping\"}\n");
        let mut long = Cursor::new(vec![b'x'; MAX_LINE + 10]);
        assert!(read_line(&mut long).is_err());
        let mut bad = Cursor::new(vec![0xFF, b'\n']);
        assert!(read_line(&mut bad).is_err());
        assert_eq!(read_line(&mut Cursor::new(Vec::new())).unwrap(), None);
    }

    /// End to end over a real socket, with a fake UI thread answering.
    #[test]
    fn tcp_round_trip() {
        let (addr, rx) = serve(0, Arc::new(|| {})).unwrap();
        std::thread::spawn(move || {
            let mut app = App::default();
            for p in rx {
                match handle(&mut app, &p.request) {
                    Ok(Action::Reply(v)) => p.answer(Ok(v)),
                    Ok(Action::Screenshot { .. }) => p.answer(Ok(json!("shot"))),
                    Err(e) => p.answer(Err(e)),
                }
            }
        });
        let mut conn = TcpStream::connect(addr).unwrap();
        writeln!(
            conn,
            r#"{{"id":1,"cmd":"key.down","params":{{"code":"KeyQ"}}}}"#
        )
        .unwrap();
        writeln!(conn, "not json").unwrap();
        writeln!(conn, r#"{{"id":3,"cmd":"state"}}"#).unwrap();
        let mut lines = BufReader::new(conn).lines();
        let first: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        assert_eq!(first, json!({"id":1,"ok":true,"result":null}));
        let second: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        assert_eq!(second["ok"], false);
        let third: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
        assert_eq!(third["result"]["test"]["presses"]["KeyQ"], 1);
    }

    proptest! {
        /// Any line from the network: parse/handle never panic.
        #[test]
        fn any_line(line in ".{0,200}", cmd in prop_oneof![
            Just("state"), Just("commands"), Just("ui.screenshot"), Just("key.down"), Just("x")
        ], params in ".{0,60}") {
            let mut app = App::default();
            if let Ok(req) = parse(&line) {
                let _ = handle(&mut app, &req);
            }
            let line = format!(r#"{{"cmd":"{cmd}","params":{params}}}"#);
            if let Ok(req) = parse(&line) {
                let _ = handle(&mut app, &req);
            }
        }
    }
}
