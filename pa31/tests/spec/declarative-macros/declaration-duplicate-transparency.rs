#![feature(decl_macro,rustc_attrs)]
#[rustc_macro_transparency="opaque"]#[rustc_macro_transparency="semiopaque"]pub macro unused(){7u32}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
