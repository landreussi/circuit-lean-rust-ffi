use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Resolve the Lean toolchain root via `lean --print-prefix`
    let lean_prefix = {
        let home = env::var("HOME").unwrap();
        let lean = PathBuf::from(&home).join(".elan/bin/lean");
        let out = Command::new(&lean)
            .arg("--print-prefix")
            .output()
            .expect("failed to run `lean --print-prefix`");
        PathBuf::from(String::from_utf8(out.stdout).unwrap().trim().to_string())
    };

    let lean_include = lean_prefix.join("include");
    let lean_lib = lean_prefix.join("lib/lean");
    let lean_syslib = lean_prefix.join("lib");

    // The Lean project's static library (built by `lake build ArithCircuit:static`)
    let project_lib = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../lean/.lake/build/lib");

    // ── Compile the C shim ──────────────────────────────────────
    cc::Build::new()
        .file("shim/arith_shim.c")
        .include(&lean_include)
        .opt_level(2)
        .compile("arith_shim");

    // ── Link order matters for static libraries ─────────────────
    // Our project library
    println!("cargo:rustc-link-search=native={}", project_lib.display());
    println!("cargo:rustc-link-lib=static=arith__circuit_ArithCircuit");

    // Lean standard libraries
    println!("cargo:rustc-link-search=native={}", lean_lib.display());
    println!("cargo:rustc-link-lib=static=Std");
    println!("cargo:rustc-link-lib=static=Init");
    println!("cargo:rustc-link-lib=static=leanrt");

    // System libraries bundled with Lean
    println!("cargo:rustc-link-search=native={}", lean_syslib.display());
    println!("cargo:rustc-link-lib=static=gmp");
    println!("cargo:rustc-link-lib=static=uv");
    println!("cargo:rustc-link-lib=static=c++");
    println!("cargo:rustc-link-lib=static=c++abi");
    println!("cargo:rustc-link-lib=static=unwind");

    // System libs that the above depend on
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=pthread");

    // Rebuild triggers
    println!("cargo:rerun-if-changed=shim/arith_shim.c");
    println!(
        "cargo:rerun-if-changed={}",
        project_lib.join("libarith__circuit_ArithCircuit.a").display()
    );
}
