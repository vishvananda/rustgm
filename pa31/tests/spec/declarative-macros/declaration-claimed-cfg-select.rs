#![feature(decl_macro,rustc_attrs)]
#[rustc_builtin_macro]pub macro cfg_select($($tt:tt)*){/* compiler built-in */}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
