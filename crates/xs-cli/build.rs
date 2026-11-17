//! The `xcreenshot` binary carries the Visual C++ runtime inside (APP-028): Windows
//! without the Visual C++ Redistributable has no `vcruntime140.dll`, and the CLI (the
//! MCP server, the installer's PATH step) did not start there, without a word. Static
//! vcruntime with the system's UCRT, the way the app does it through `tauri-build`
//! (its `static_vcruntime.rs`, MIT/Apache-2.0, after ChrisDenton/static_vcruntime).

use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if !(windows && msvc) {
        return;
    }
    override_msvcrt_lib();
    for lib in [
        "libvcruntimed.lib",
        "vcruntime.lib",
        "vcruntimed.lib",
        "libcmtd.lib",
        "msvcrt.lib",
        "msvcrtd.lib",
        "libucrt.lib",
        "libucrtd.lib",
    ] {
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:{lib}");
    }
    for lib in ["libcmt.lib", "libvcruntime.lib", "ucrt.lib"] {
        println!("cargo:rustc-link-arg=/DEFAULTLIB:{lib}");
    }
}

/// Rust names `msvcrt.lib` itself; an almost empty one of the same name in the output
/// folder takes its place.
fn override_msvcrt_lib() {
    let machine: &[u8] = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => &[0x64, 0x86],
        Ok("x86") => &[0x4C, 0x01],
        _ => return,
    };
    let bytes: &[u8] = &[
        1, 0, 94, 3, 96, 98, 60, 0, 0, 0, 1, 0, 0, 0, 0, 0, 132, 1, 46, 100, 114, 101, 99, 116, 118, 101, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 16, 0, 46,
        100, 114, 101, 99, 116, 118, 101, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 4, 0, 0, 0,
    ];
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let path = Path::new(&out_dir).join("msvcrt.lib");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        f.write_all(machine).expect("msvcrt.lib");
        f.write_all(bytes).expect("msvcrt.lib");
    }
    println!("cargo:rustc-link-search=native={out_dir}");
}
