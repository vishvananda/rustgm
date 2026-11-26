#![feature(no_core)]
#![no_core]

extern crate base;

pub struct Local;

unsafe impl base::Applicable for Local {}
