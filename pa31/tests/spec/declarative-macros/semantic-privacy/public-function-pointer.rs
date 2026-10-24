#![feature(decl_macro)]
mod provider {fn helper()->u32{7} pub fn expose()->fn()->u32{helper} pub macro value(){expose()()} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{provider::value!()}
