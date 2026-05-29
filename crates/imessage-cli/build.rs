fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let plist = concat!(env!("CARGO_MANIFEST_DIR"), "/Info.plist");
        println!("cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{plist}");
    }
}
