#![feature(no_core)]
#![no_core]
extern crate prov;
use prov::builtin::concat;
#[unsafe(no_mangle)]
pub extern "C" fn invoked_line() -> u32 {
    prov::line!()
}
#[unsafe(no_mangle)]
pub extern "C" fn invoked_column() -> u32 {
    let _joined: &str = concat!("a", 1);
    let _spelled: &str = prov::stringify!(b c);
    let _file: &str = prov::builtin::file!();
    let _module: &str = prov::builtin::deeper::module_path!();
        prov::column!()
}
#[unsafe(no_mangle)]
pub extern "C" fn configured() -> bool {
    prov::cfg!(target_os = "linux")
}
