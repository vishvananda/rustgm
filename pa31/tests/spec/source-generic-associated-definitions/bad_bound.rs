#![feature(no_core)]
#![no_core]

extern crate base;

struct NotElement;

pub fn bad(_: <base::Lane<u32, 4> as base::Castable>::Cast<NotElement>) {}
