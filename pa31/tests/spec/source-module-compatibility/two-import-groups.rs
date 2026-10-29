#![feature(no_core)]
#![no_core]
extern crate base;
use base::gated::{marked as first, plain as second};
pub fn probe()->u8{first()}
