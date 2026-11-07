#![feature(no_core)]
#![no_core]
extern crate prov;
prov::compile_error!("builtin row reached the consumer expander");
pub fn unused() -> u32 {
    1
}
