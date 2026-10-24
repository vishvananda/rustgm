#![feature(decl_macro)]
#[rustc_macro_transparency="opaque"]pub macro unused(){7u32}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
