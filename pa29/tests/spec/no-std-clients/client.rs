#![no_std]
// `Option` and `forty_two!` come from the injected edition prelude, `base`
// through the injected `core` extern-prelude entry.
fn pick(value: Option<u32>) -> u32 {
    match value {
        Option::Some(_) => forty_two!(),
        Option::None => 0,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_prelude() -> u32 {
    pick(Option::Some(core::base()))
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_path() -> u32 {
    ::core::base()
}
