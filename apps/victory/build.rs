fn main() {
    println!("cargo::rerun-if-changed=agility.def");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if env == "msvc" {
        println!("cargo::rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo::rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
    } else if env == "gnu" {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo::rustc-link-arg-bins={dir}/agility.def");
    }
}
