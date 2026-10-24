#![feature(decl_macro)]
pub macro unused{()=>{7u32};()=>{8u32}}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
