#![feature(no_core, lexical_function)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn local_constant() -> u8 {
    base::exposed()
}

#[unsafe(no_mangle)]
pub extern "C" fn checked_constructor(value: u8) -> u8 {
    base::Mark(value).0
}

#[unsafe(no_mangle)]
pub extern "C" fn stable_first() -> u8 {
    base::Marker::First as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn stable_second() -> u8 {
    base::Marker::Second as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn ordinary_trait_member() -> u8 {
    <base::Mark as base::Evaluation>::runtime()
}

#[unsafe(no_mangle)]
pub extern "C" fn nested_initializer() -> u8 {
    base::EXPOSED
}

#[unsafe(no_mangle)]
pub extern "C" fn derived_implementation() -> u8 {
    let input = base::Derived;
    let value = <base::Derived as base::Clone>::clone(&input);
    value.value()
}
