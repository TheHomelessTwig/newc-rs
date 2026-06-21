fn main() {
    // Embeds FILEVERSION/PRODUCTVERSION (from CARGO_PKG_VERSION) and basic
    // metadata into the Windows .exe's VERSIONINFO resource, so tools like
    // `Get-Command`/file properties show the real version instead of
    // 0.0.0.0. No-op on every other target.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "newc - C project scaffolding and management tool");
        res.set("ProductName", "newc");
        res.set("LegalCopyright", "MIT License");
        res.compile().expect("failed to embed Windows version resource");
    }
}
