fn unused_helper() {}
#[cfg(host_lint_probe)]
fn guarded_helper() {}
fn main() { println!("cargo::rustc-cfg=host_lint_checked"); }
