#![feature(no_core)]
#![no_core]
extern crate base;

const DIRECT: u32 = base::chained();
const FORWARDED: u32 = base::forwarded();
const FIRST: u32 = base::generic::<37>();
const SECOND: u32 = base::generic::<41>();
const METHOD: u32 = base::Wrap(23).value();

#[unsafe(no_mangle)]
pub extern "C" fn obs_direct() -> u32 { DIRECT }

#[unsafe(no_mangle)]
pub extern "C" fn obs_forwarded() -> u32 { FORWARDED }

#[unsafe(no_mangle)]
pub extern "C" fn obs_first() -> u32 { FIRST }

#[unsafe(no_mangle)]
pub extern "C" fn obs_second() -> u32 { SECOND }

#[unsafe(no_mangle)]
pub extern "C" fn obs_method() -> u32 { METHOD }

#[unsafe(no_mangle)]
pub extern "C" fn obs_ordinary() -> u32 { base::seed(19) }
