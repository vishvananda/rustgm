#![feature(no_core)]
#![no_core]
#![feature(controlling_api, current_api)]
extern crate base;


#[unsafe(no_mangle)]
pub extern "C" fn read_implied() -> u8 { base::dependent() }

#[unsafe(no_mangle)]
pub extern "C" fn read_renamed() -> u8 { base::renamed() }

