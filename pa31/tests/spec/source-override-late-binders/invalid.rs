#![feature(no_core)]
#![no_core]
extern crate base;
use base::PartialEq;
pub fn wrong_rhs()->bool { base::Wrap(1).eq(&true) }
