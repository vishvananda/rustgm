#![feature(no_core)]
#![no_core]
extern crate base;

const SITE: &base::Location<'static> = base::Location::caller();

#[unsafe(no_mangle)]
pub extern "C" fn direct_line() -> u32 {
    base::Location::caller().line()
}

#[unsafe(no_mangle)]
pub extern "C" fn direct_column() -> u32 {
    base::Location::caller().column()
}

#[unsafe(no_mangle)]
pub extern "C" fn folded_line() -> u32 { SITE.line() }

#[unsafe(no_mangle)]
pub extern "C" fn folded_column() -> u32 { SITE.column() }
