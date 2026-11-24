#![feature(no_core)]
#![no_core]
extern crate base;

const BAD: u8 = base::Signal::<base::Payload<u16>, 4>::FLAG;
