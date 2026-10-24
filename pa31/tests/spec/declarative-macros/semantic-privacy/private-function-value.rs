#![feature(decl_macro)]
mod provider { fn helper()->u32 {7} pub macro value() {helper} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { let f=provider::value!(); f() }
