//! Post-build flash artifacts for tool-agnostic SWD programming.
//!
//! - MCU: STM32H745BIT6 (Cortex-M7)
//! - Flash base: 0x08000000
//! - SWD: SWDIO, SWCLK, GND (optional NRST)
//!
//! Hardware release build (no QEMU semihosting):
//!   cargo hw
//!   # or: cargo build --release --no-default-features

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target != "thumbv7em-none-eabihf" {
        return;
    }

    let linker = if env::var("CARGO_FEATURE_QEMU").is_ok() {
        "linker-qemu.ld"
    } else {
        "linker.ld"
    };
    println!("cargo:rustc-link-arg=-T{linker}");
    println!("cargo:rerun-if-changed={linker}");
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=linker-qemu.ld");

    let profile = env::var("PROFILE").unwrap();
    if profile != "release" {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let elf = out_dir
        .ancestors()
        .nth(3)
        .expect("profile dir")
        .join("ArmOS");

    if !elf.exists() {
        println!("cargo:warning=ELF not found at {} — skipping objcopy", elf.display());
        return;
    }

    let bin = elf.with_extension("bin");
    let hex = elf.with_extension("hex");

    objcopy(&elf, &bin, &["-O", "binary"]);
    objcopy(&elf, &hex, &["-O", "ihex"]);

    println!("cargo:rerun-if-changed=build.rs");
}

fn objcopy(elf: &PathBuf, out: &PathBuf, extra_args: &[&str]) {
    let llvm_objcopy = find_llvm_tool("llvm-objcopy");
    let gnu_objcopy = "objcopy";

    let status = Command::new(&llvm_objcopy)
        .arg(elf)
        .args(extra_args)
        .arg(out)
        .status();

    if status.map(|s| s.success()).unwrap_or(false) {
        println!(
            "cargo:warning=Generated {} (flash at 0x08000000)",
            out.display()
        );
        return;
    }

    let status = Command::new(gnu_objcopy)
        .args(extra_args)
        .arg(elf)
        .arg(out)
        .status()
        .expect("failed to run objcopy");

    if !status.success() {
        panic!("objcopy failed for {}", out.display());
    }

    println!(
        "cargo:warning=Generated {} (flash at 0x08000000)",
        out.display()
    );
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
    let tool = PathBuf::from(sysroot.trim())
        .join("lib/rustlib")
        .join(env::var("HOST").unwrap())
        .join("bin")
        .join(name);

    if tool.exists() {
        tool.to_string_lossy().into_owned()
    } else {
        name.to_string()
    }
}