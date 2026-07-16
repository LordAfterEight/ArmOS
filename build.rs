//! Build script: linker selection, flash artifacts, ensure custom QEMU.
//!
//! | Binary       | Linker           | Placement                          |
//! |--------------|------------------|------------------------------------|
//! | `ArmOS`      | `linker.ld`      | NOR XIP @ 0x90000000, SDRAM RAM    |
//! | `bootloader` | `linker-boot.ld` | MCU flash @ 0x08000000, DTCM RAM   |
//! | `mps2` feat. | `linker-qemu.ld` | Legacy mps2-an500                  |

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    if target != "thumbv7em-none-eabihf" {
        return;
    }

    let mps2 = env::var("CARGO_FEATURE_MPS2").is_ok();

    // Per-binary link args: a package-wide `-T` would apply the same script to every bin.
    if mps2 {
        println!("cargo:rustc-link-arg-bins=-Tlinker-qemu.ld");
    } else {
        println!("cargo:rustc-link-arg-bin=ArmOS=-Tlinker.ld");
        println!("cargo:rustc-link-arg-bin=bootloader=-Tlinker-boot.ld");
    }
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=linker-boot.ld");
    println!("cargo:rerun-if-changed=linker-qemu.ld");
    println!("cargo:rerun-if-changed=qemu-stm32h745/patches");
    println!("cargo:rerun-if-changed=qemu-stm32h745/scripts/ensure-qemu.sh");

    if !mps2 && env::var("ARMOS_SKIP_QEMU_BUILD").is_err() {
        ensure_custom_qemu(&manifest_dir);
    }

    let profile = env::var("PROFILE").unwrap();
    if profile != "release" {
        return;
    }

    // Emit .bin/.hex for binaries in this profile dir (best-effort).
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile_dir = out_dir.ancestors().nth(3).expect("profile dir");
    let name = env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "ArmOS".into());
    let elf = profile_dir.join(&name);

    if !elf.exists() {
        return;
    }

    let bin = elf.with_extension("bin");
    let hex = elf.with_extension("hex");
    objcopy(&elf, &bin, &["-O", "binary"]);
    objcopy(&elf, &hex, &["-O", "ihex"]);
    println!("cargo:rerun-if-changed=build.rs");
}

fn ensure_custom_qemu(manifest_dir: &std::path::Path) {
    let script = manifest_dir.join("qemu-stm32h745/scripts/ensure-qemu.sh");
    if !script.exists() {
        println!(
            "cargo:warning=qemu-stm32h745 missing (no {}); skip QEMU ensure",
            script.display()
        );
        return;
    }

    println!("cargo:warning=ensuring stm32h745-carrier QEMU (first build may take a while)…");
    let status = Command::new("bash").arg(&script).status();
    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=custom QEMU ready");
        }
        Ok(s) => {
            println!("cargo:warning=ensure-qemu.sh failed (status {s})");
        }
        Err(e) => {
            println!("cargo:warning=could not run ensure-qemu.sh: {e}");
        }
    }
}

fn objcopy(elf: &PathBuf, out: &PathBuf, extra_args: &[&str]) {
    for tool in [
        find_llvm_tool("rust-objcopy"),
        find_llvm_tool("llvm-objcopy"),
        "llvm-objcopy".into(),
        "arm-none-eabi-objcopy".into(),
        "objcopy".into(),
    ] {
        let status = Command::new(&tool)
            .arg(elf)
            .args(extra_args)
            .arg(out)
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            println!("cargo:warning=Generated {}", out.display());
            return;
        }
    }
}

fn find_llvm_tool(name: &str) -> String {
    if Command::new(name)
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return name.to_string();
    }

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(&rustc)
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("rustc --print sysroot");

    let sysroot = String::from_utf8(output.stdout).expect("sysroot utf8");
    let host = env::var("HOST").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".into());
    let tool = PathBuf::from(sysroot.trim())
        .join("lib/rustlib")
        .join(host)
        .join("bin")
        .join(name);

    if tool.exists() {
        tool.to_string_lossy().into_owned()
    } else {
        name.to_string()
    }
}
