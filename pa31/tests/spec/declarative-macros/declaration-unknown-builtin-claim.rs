#![feature(decl_macro,rustc_attrs)]
#[rustc_builtin_macro(not_a_real_builtin)]pub macro unused(){7u32}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
