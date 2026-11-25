use std::path::PathBuf;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        println!("cargo::rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo::rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
    } else if target_env == "gnu" {
        let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
        let def = out_dir.join("agility.def");
        std::fs::write(&def, "EXPORTS\n    D3D12SDKVersion DATA\n    D3D12SDKPath DATA\n")
            .expect("write agility.def");
        println!("cargo::rustc-link-arg-bins={}", def.display());
    }
}
