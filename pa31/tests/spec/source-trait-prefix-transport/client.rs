#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;
use base::{Child, Marker, Parent};
struct Item(u8);
struct Extra(u8);
impl Parent for Item {
    type Output = u8;
    fn produce(&self) -> u8 { self.0 }
}
impl Marker for Extra {
    fn score(&self) -> u8 { self.0 }
}
impl Child for Item { type Other = Extra; }
fn inspect<T: Child>(value: &T, extra: &T::Other) -> u8 {
    match value.produce() { 1 => extra.score(), other => other }
}
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 { inspect(&Item(1), &Extra(41)) }
