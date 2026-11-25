#![feature(no_core)]
#![no_core]
extern crate base;

const EVALUATED: u32 = base::generic::<17>();

#[unsafe(no_mangle)]
pub extern "C" fn obs_ordinary() -> u32 { base::ordinary() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_const() -> u32 { EVALUATED }

#[unsafe(no_mangle)]
pub extern "C" fn obs_generic() -> u32 { base::generic::<29>() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_closure() -> u32 { base::in_closure::<11>() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_nested() -> u32 { base::nested() }

#[unsafe(no_mangle)]
pub extern "C" fn obs_initializer() -> u32 { base::VALUE }

const TYPE_EVALUATED: u32 = base::for_type::<u8>();

#[unsafe(no_mangle)]
pub extern "C" fn obs_type_const() -> u32 { TYPE_EVALUATED }

#[unsafe(no_mangle)]
pub extern "C" fn obs_type_runtime() -> u32 { base::for_type::<u32>() }
