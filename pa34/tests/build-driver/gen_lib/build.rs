use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    fs::write(
        out_dir.join("generated.rs"),
        "pub const GENERATED_BASE: i32 = 42;\n",
    )
    .expect("write generated.rs");
    println!("cargo::rustc-cfg=generated_mode");
    println!("cargo::rustc-env=PA34_TAG=build_ok");
}
