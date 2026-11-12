// Embed the .exe icon + version info when building ON Windows (GitHub Actions windows runner).
// Cross-builds from Linux skip this (the window/taskbar icon is still set at runtime).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon/icon.ico");
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/icon/icon.ico");
            res.set("ProductName", "TikTok Shrine Live");
            res.set("FileDescription", "TikTok Shrine Live - ไหว้พระขอเลข / เซียมซี");
            if let Err(e) = res.compile() {
                println!("cargo:warning=could not embed icon: {e}");
            }
        }
    }
}
