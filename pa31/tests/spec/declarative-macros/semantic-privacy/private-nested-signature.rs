#![feature(decl_macro)]
mod provider { struct Hidden; pub fn take(_:fn()->(Hidden,)) {} pub macro value() {take} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 {let _=provider::value!();7}
