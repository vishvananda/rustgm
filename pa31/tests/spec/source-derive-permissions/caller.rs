#![feature(no_core, location_fixture)]
#![no_core]
extern crate base as core;

pub fn authored_type(_: core::clone::AssertParamIsClone<u8>) {}
