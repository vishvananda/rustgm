#![feature(no_core)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn identity(value: u32) -> u32 {
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_unit_const() -> u32 {
    <()>::UNIT_CONST
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_unit_method() -> u32 {
    let unit = ();
    unit.unit_tag()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_tuple_method() -> u32 {
    let pair = (29u32,);
    *pair.first_ref()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_ref_method() -> u32 {
    let value = 41u32;
    let shared: &u32 = &value;
    *shared.reborrow_tag()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_mut_ref_method() -> u32 {
    let mut value = 7u32;
    let mut slot: &mut u32 = &mut value;
    slot.write_copy(53);
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_fn_ptr() -> u32 {
    let f: fn(u32) -> u32 = identity;
    f(6)
}
