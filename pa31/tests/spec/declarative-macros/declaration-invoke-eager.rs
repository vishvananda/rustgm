#![feature(decl_macro)]
pub macro selected(){"unused"}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{let _=concat!(selected!());7u32}
