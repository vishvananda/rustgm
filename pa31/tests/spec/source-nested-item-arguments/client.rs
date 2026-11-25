#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn obs_tuple() -> u32 { base::tuple_items() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_nominal() -> u32 { base::nominal_items() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_array() -> u32 { base::array_items() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_reference() -> u32 { base::reference_items() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_lifetime() -> u32 {
    let x = 77u32;
    let y = 88u32;
    *base::choose_refs(&x, &y, true)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_other_item() -> u32 { base::tuple_second() }
