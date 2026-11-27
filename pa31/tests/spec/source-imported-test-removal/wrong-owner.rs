#![feature(no_core, role_fixture)]
#![no_core]
extern crate base;

#[base::test]
pub struct WrongOwner;
