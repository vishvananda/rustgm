#![feature(no_core,module_identity)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{base::gated::forwarded(7u8)}
