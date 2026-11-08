// A no_core client names core through `--extern core=`, as the
// rustc_std_workspace_core shim does.
#![feature(no_core)]
#![no_core]
pub fn base() -> u32 {
    core::base()
}
