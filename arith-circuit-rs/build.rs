use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let lean_prefix = {
        let path = Command::new("which").arg("lean").output().expect("lean isn't installed").stdout;
        let lean = String::from_utf8(path).unwrap().trim().to_string();
        let out = Command::new(&lean)
            .arg("--print-prefix")
            .output()
            .expect("failed to run `lean --print-prefix`");
        PathBuf::from(String::from_utf8(out.stdout).unwrap().trim().to_string())
    };

    let lean_include = lean_prefix.join("include");
    let lean_lib = lean_prefix.join("lib/lean");
    let lean_syslib = lean_prefix.join("lib");

    // Our project's static library (built by `lake build ArithCircuit:static`)
    let project_lib = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../lean/.lake/build/lib");

    let libuv = pkg_config::Config::new()
        .cargo_metadata(false)
        .probe("libuv")
        .expect("libuv or pkg-config isn't installed")
        .link_paths
        .pop()
        .expect("libuv: couldnt find any link path");
    
    // changed between 4.23 and 4.29, so we can't use lean-sys's version).
    cc::Build::new()
        .file("shim/closure_shim.c")
        .include(&lean_include)
        .opt_level(2)
        .compile("closure_shim");

    println!("cargo:rustc-link-search=native={}", project_lib.display());
    println!("cargo:rustc-link-search=native={}", libuv.display());
    println!("cargo:rustc-link-lib=static=arith__circuit_ArithCircuit");

    // Libs that lean-sys doesn't link but Lean 4.29 needs
    println!("cargo:rustc-link-search=native={}", lean_lib.display());
    println!("cargo:rustc-link-lib=static=Std");

    println!("cargo:rustc-link-search=native={}", lean_syslib.display());

    println!("cargo:rustc-link-lib=static=c++");
    println!("cargo:rustc-link-lib=static=c++abi");
    println!("cargo:rustc-link-lib=static=unwind");

    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-arg=-luv");

    println!("cargo:rerun-if-changed=shim/closure_shim.c");
    println!(
        "cargo:rerun-if-changed={}",
        project_lib.join("libarith__circuit_ArithCircuit.a").display()
    );
}
