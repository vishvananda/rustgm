#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn read_direct() -> u8 { base::old!() }

#[unsafe(no_mangle)]
pub extern "C" fn read_alias() -> u8 { base::old_alias!() }

#[unsafe(no_mangle)]
pub extern "C" fn read_scheduled() -> u8 { base::later!() }

#[unsafe(no_mangle)]
pub extern "C" fn read_modern() -> u8 { base::modern_alias() }

#[unsafe(no_mangle)]
pub extern "C" fn read_legacy() -> u8 { base::legacy_alias() }
