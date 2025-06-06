fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.compile().unwrap();
    }
}
