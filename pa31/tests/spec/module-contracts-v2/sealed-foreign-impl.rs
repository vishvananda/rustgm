#![feature(no_core)]#![no_core]
extern crate base; struct Local; impl base::num::Sealed for Local {}
