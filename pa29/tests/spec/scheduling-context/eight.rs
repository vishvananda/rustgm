#![feature(no_core)]
#![no_core]
extern crate base;
const C0: usize = base::four();
fn take0(_: &[u8; C0]) -> i32 { 42 }
const C1: usize = base::four();
fn take1(_: &[u8; C1]) -> i32 { 42 }
const C2: usize = base::four();
fn take2(_: &[u8; C2]) -> i32 { 42 }
const C3: usize = base::four();
fn take3(_: &[u8; C3]) -> i32 { 42 }
const C4: usize = base::four();
fn take4(_: &[u8; C4]) -> i32 { 42 }
const C5: usize = base::four();
fn take5(_: &[u8; C5]) -> i32 { 42 }
const C6: usize = base::four();
fn take6(_: &[u8; C6]) -> i32 { 42 }
const C7: usize = base::four();
fn take7(_: &[u8; C7]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take0(&[1, 2, 3, 4]) }
