#![feature(no_core)]
#![no_core]
extern crate base;
pub fn escape<'a>(slot: &mut &'a u32) {
    let local=99;
    base::choose(slot,&local,false);
}
