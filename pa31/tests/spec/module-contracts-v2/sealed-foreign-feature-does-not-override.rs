#![feature(no_core,impl_restriction)]#![no_core]
extern crate base; struct Local; impl base::num::Sealed for Local {}
