#![feature(no_core)]
#![no_core]
#![deny(deprecated, unfulfilled_lint_expectations)]
extern crate base;

pub fn current_binding() -> u8 { base::modern_alias() }

#[allow(deprecated)]
pub fn allowed() -> u8 { base::old!() }

#[expect(deprecated)]
pub fn expected() -> u8 { base::old_alias!() }

pub fn still_current() -> u8 { base::later!() }
