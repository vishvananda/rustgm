#![feature(decl_macro,rustc_attrs)]
#[rustc_macro_transparency="semiopaque"]pub macro unused($value:expr){not_resolved($value)}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
