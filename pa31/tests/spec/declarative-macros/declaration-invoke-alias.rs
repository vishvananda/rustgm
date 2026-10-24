#![feature(decl_macro)]
use owner::selected as alias;mod owner{pub macro selected(){7u32}}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{alias!()}
