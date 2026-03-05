//! Windows: embeds `assets/keyprobe.ico` (made by `assets/icon.py`) as the `.exe` icon.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/keyprobe.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/keyprobe.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=keyprobe.ico not embedded: {e}");
        }
    }
}
