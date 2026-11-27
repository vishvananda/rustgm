#![feature(no_core)]
#![no_core]
extern crate base;

pub fn invalid() -> bool {
    if base::forward!(#[allow(unused)] true) {
        true
    } else {
        false
    }
}
