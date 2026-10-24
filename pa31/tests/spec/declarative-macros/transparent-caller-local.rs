#![feature(decl_macro)]
#![feature(rustc_attrs)]
#[rustc_macro_transparency = "transparent"] macro value() { local }
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { let local = 19u32; value!() }
