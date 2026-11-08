#![no_std]
// A variant is not a type: rustc rejects it in type position (E0573).
pub fn take(_value: None) {}
