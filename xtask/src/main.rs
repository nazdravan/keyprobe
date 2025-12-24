//! Repo automation. Run as `cargo xtask <task>`.
//!
//! - `ci`     — the gate before every commit: fmt, clippy, test, layers, wasm.
//! - `layers` — every crate declares `[package.metadata.layer] level`; a crate may only depend on
//!   lower layers, and nothing below L6 may (even transitively) pull in a UI/windowing crate.
//! - `wasm`   — `cargo clippy -D warnings` for wasm32 of every crate that must run on the web
//!   (level <= 6 unless the crate sets `[package.metadata.layer] wasm = false`, plus any crate
//!   that sets `wasm = true`), so wasm-only code is linted too.
//! - `web`    — release build of `keyprobe-web` + `wasm-bindgen` → `dist/` (open via a static
//!   server, e.g. `python3 -m http.server -d dist`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::process::{Command, ExitCode};

use serde_json::Value;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
/// Crates that must stay in L6 and above.
const UI_CRATES: &[&str] = &[
    "egui",
    "eframe",
    "egui-winit",
    "egui-wgpu",
    "wgpu",
    "winit",
    "rfd",
];

type Result<T = ()> = std::result::Result<T, String>;

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "ci" => ci(),
        "fmt" => fmt(),
        "clippy" => clippy(),
        "test" => test(),
        "layers" => layers(),
        "wasm" => wasm(),
        "web" => web(),
        _ => Err(format!(
            "unknown task {task:?}; expected one of: ci, fmt, clippy, test, layers, wasm, web"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask {task}: FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

fn ci() -> Result {
    for (name, step) in [
        ("fmt", fmt as fn() -> Result),
        ("clippy", clippy),
        ("test", test),
        ("layers", layers),
        ("wasm", wasm),
    ] {
        eprintln!("== xtask {name}");
        step().map_err(|e| format!("{name}: {e}"))?;
    }
    eprintln!("== ci: all checks passed");
    Ok(())
}

fn fmt() -> Result {
    cargo(&["fmt", "--all", "--check"])
}

fn clippy() -> Result {
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])
}

fn test() -> Result {
    cargo(&["test", "--workspace", "--quiet"])
}

fn wasm() -> Result {
    let meta = metadata()?;
    let crates: Vec<String> = members(&meta)?
        .into_iter()
        .filter(|m| m.wasm.unwrap_or(m.level <= 6))
        .map(|m| m.name)
        .collect();
    let mut args = vec!["clippy", "--quiet", "--target", WASM_TARGET];
    for name in &crates {
        args.extend(["-p", name.as_str()]);
    }
    args.extend(["--", "-D", "warnings"]);
    cargo(&args).map_err(|e| {
        format!("{e} (is the {WASM_TARGET} target installed? use rustup's cargo, see CLAUDE.md)")
    })
}

const WEB_CRATE: &str = "keyprobe-web";
const DIST: &str = "dist";

fn web() -> Result {
    cargo(&[
        "build",
        "--quiet",
        "--release",
        "--target",
        WASM_TARGET,
        "-p",
        WEB_CRATE,
    ])?;
    let meta = metadata()?;
    let target_dir = at(&meta, "/target_directory")
        .as_str()
        .ok_or("cargo metadata has no target_directory")?;
    let root = at(&meta, "/workspace_root")
        .as_str()
        .ok_or("cargo metadata has no workspace_root")?;
    let wasm = format!("{target_dir}/{WASM_TARGET}/release/keyprobe_web.wasm");
    let dist = format!("{root}/{DIST}");
    let status = Command::new("wasm-bindgen")
        .args(["--target", "web", "--no-typescript", "--out-dir", &dist, &wasm])
        .status()
        .map_err(|e| {
            format!("wasm-bindgen: {e} (install: cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock>)")
        })?;
    if !status.success() {
        return Err(format!("wasm-bindgen exited with {status}"));
    }
    std::fs::copy(
        format!("{root}/apps/keyprobe-web/index.html"),
        format!("{dist}/index.html"),
    )
    .map_err(|e| format!("copy index.html: {e}"))?;
    eprintln!("web: built {dist}/ (serve it: python3 -m http.server -d {DIST})");
    Ok(())
}

struct Member {
    id: String,
    name: String,
    level: u64,
    wasm: Option<bool>,
}

fn layers() -> Result {
    let meta = metadata()?;
    let members = members(&meta)?;
    let by_id: BTreeMap<&str, &Member> = members.iter().map(|m| (m.id.as_str(), m)).collect();
    let graph = normal_deps(&meta)?;
    let names = package_names(&meta);
    let mut errors = Vec::new();

    for m in &members {
        let direct = graph.get(m.id.as_str()).cloned().unwrap_or_default();
        for dep in &direct {
            if let Some(d) = by_id.get(dep.as_str())
                && d.level >= m.level
            {
                errors.push(format!(
                    "{} (L{}) depends on {} (L{}): only lower layers allowed",
                    m.name, m.level, d.name, d.level
                ));
            }
        }
        if m.level < 6 {
            // Breadth-first over normal dependencies, remembering the path for the message.
            let mut seen = BTreeSet::new();
            let mut queue: VecDeque<(String, String)> =
                direct.into_iter().map(|d| (d, m.name.clone())).collect();
            while let Some((id, path)) = queue.pop_front() {
                if !seen.insert(id.clone()) {
                    continue;
                }
                let name = names.get(id.as_str()).map(String::as_str).unwrap_or("?");
                let path = format!("{path} -> {name}");
                if UI_CRATES.contains(&name) {
                    errors.push(format!("L{} crate pulls in UI crate: {path}", m.level));
                    continue;
                }
                for next in graph.get(id.as_str()).cloned().unwrap_or_default() {
                    queue.push_back((next, path.clone()));
                }
            }
        }
    }

    if errors.is_empty() {
        eprintln!("layers: {} crates OK", members.len());
        Ok(())
    } else {
        Err(errors.join("\n  "))
    }
}

fn metadata() -> Result<Value> {
    let out = Command::new(cargo_bin())
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata json: {e}"))
}

fn members(meta: &Value) -> Result<Vec<Member>> {
    let ids: BTreeSet<&str> = at(meta, "/workspace_members")
        .as_array()
        .ok_or("no workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages = at(meta, "/packages").as_array().ok_or("no packages")?;
    let mut out = Vec::new();
    for p in packages {
        let id = at(p, "/id").as_str().unwrap_or_default();
        if !ids.contains(id) {
            continue;
        }
        let name = at(p, "/name").as_str().unwrap_or_default().to_owned();
        let layer = &at(p, "/metadata/layer");
        let level = at(layer, "/level").as_u64().ok_or_else(|| {
            format!("{name}: missing [package.metadata.layer] level = N in Cargo.toml")
        })?;
        out.push(Member {
            id: id.to_owned(),
            name,
            level,
            wasm: at(layer, "/wasm").as_bool(),
        });
    }
    Ok(out)
}

fn package_names(meta: &Value) -> BTreeMap<String, String> {
    at(meta, "/packages")
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| {
            Some((
                at(p, "/id").as_str()?.to_owned(),
                at(p, "/name").as_str()?.to_owned(),
            ))
        })
        .collect()
}

/// Package id -> ids of its normal (non-dev, non-build) dependencies.
fn normal_deps(meta: &Value) -> Result<BTreeMap<String, Vec<String>>> {
    let nodes = at(meta, "/resolve/nodes")
        .as_array()
        .ok_or("cargo metadata has no resolve graph")?;
    let mut out = BTreeMap::new();
    for node in nodes {
        let Some(id) = at(node, "/id").as_str() else {
            continue;
        };
        let deps = at(node, "/deps")
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| {
                at(d, "/dep_kinds")
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|k| at(k, "/kind").is_null())
            })
            .filter_map(|d| at(d, "/pkg").as_str().map(str::to_owned))
            .collect();
        out.insert(id.to_owned(), deps);
    }
    Ok(out)
}

/// JSON pointer lookup that yields `null` instead of failing (serde_json's `[]` is flagged by clippy).
fn at<'a>(v: &'a Value, pointer: &str) -> &'a Value {
    static NULL: Value = Value::Null;
    v.pointer(pointer).unwrap_or(&NULL)
}

fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn cargo(args: &[&str]) -> Result {
    let status = Command::new(cargo_bin())
        .args(args)
        .status()
        .map_err(|e| format!("cargo {}: {e}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` exited with {status}", args.join(" ")))
    }
}
