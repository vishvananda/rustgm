#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _f: base::Tag<5> = base::five(); let _r: base::Tag<7> = base::rigid::<7>(); match base::nine().get() { 42 => match base::forward::<4>(41).get() { 41 => 42, _ => 8 }, _ => 7 } }
