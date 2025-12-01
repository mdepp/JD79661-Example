//! SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! Copyright (c) 2021–2024 The rp-rs Developers
//! Copyright (c) 2021 rp-rs organization
//! Copyright (c) 2025 Raspberry Pi Ltd.
//!
//! Set up linker scripts

use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use regex::Regex;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(rp2040)");
    println!("cargo::rustc-check-cfg=cfg(rp2350)");

    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=.pico-rs");
    let contents = read_to_string(".pico-rs")
        .map(|s| s.trim().to_string().to_lowercase())
        .unwrap_or_else(|e| {
            eprintln!("Failed to read file: {}", e);
            String::new()
        });

    // The file `memory.x` is loaded by cortex-m-rt's `link.x` script, which
    // is what we specify in `.cargo/config.toml` for Arm builds
    let target;
    if contents == "rp2040" {
        target = "thumbv6m-none-eabi";
        let memory_x = include_bytes!("rp2040.x");
        let mut f = File::create(out.join("memory.x")).unwrap();
        f.write_all(memory_x).unwrap();
        println!("cargo::rustc-cfg=rp2040");
        println!("cargo:rerun-if-changed=rp2040.x");
    } else {
        if contents.contains("riscv") {
            target = "riscv32imac-unknown-none-elf";
        } else {
            target = "thumbv8m.main-none-eabihf";
        }
        let memory_x = include_bytes!("rp2350.x");
        let mut f = File::create(out.join("memory.x")).unwrap();
        f.write_all(memory_x).unwrap();
        println!("cargo::rustc-cfg=rp2350");
        println!("cargo:rerun-if-changed=rp2350.x");
    }

    // XXX Previously the build script would use this target to modify
    // config.toml. This is probably not a good idea, especially not when other
    // packages can be using a different target. Instead, shortcircuit here if
    // we're using the wrong target.
    let configured_target = std::env::var("TARGET").unwrap();
    if configured_target != target {
        println!(
            "cargo::error=Bad target: According to .pico-rs, `simulator` must \
            be built using target `{target}`. Instead, target \
            `{configured_target}` is configured."
        )
    }

    // The file `rp2350_riscv.x` is what we specify in `.cargo/config.toml` for
    // RISC-V builds
    let rp2350_riscv_x = include_bytes!("rp2350_riscv.x");
    let mut f = File::create(out.join("rp2350_riscv.x")).unwrap();
    f.write_all(rp2350_riscv_x).unwrap();
    println!("cargo:rerun-if-changed=rp2350_riscv.x");

    let unix_time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("cargo::rustc-env=BUILD_TIMESTAMP={unix_time}");

    println!("cargo:rerun-if-changed=build.rs");
}
