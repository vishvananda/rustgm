#![feature(no_core)]
#![no_core]
extern crate prov as renamed;
// A local module with the provider's placement name binds nothing there.
pub mod builtin {}
use renamed::builtin::deeper::module_path;
pub fn spelled() -> u32 {
    let _module: &str = module_path!();
    let _joined: &str = renamed::builtin::concat!("x", 2);
    let _spelled: &str = renamed::stringify!(y);
    renamed::line!()
}
