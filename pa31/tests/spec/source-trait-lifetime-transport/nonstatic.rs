#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Borrowed<'a>(&'a u8);

impl<'a> base::StaticOnly for Borrowed<'a> {}
