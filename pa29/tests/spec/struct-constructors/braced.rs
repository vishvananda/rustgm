#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::unwrap(base::Wrap { 0: 42 }) { 42 => match base::wrap(42) { base::Wrap { 0: x } => match x { 42 => 42, _ => 8 } }, _ => 7 } }
