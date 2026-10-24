#![feature(decl_macro)]
pub macro unused($value:expr){$missing}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
