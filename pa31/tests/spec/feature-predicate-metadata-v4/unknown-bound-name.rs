#![feature(no_core,predicate_api,scope_only)]
#![no_core]
extern crate base;
pub fn answer()->u8{base::scoped(7u8)}
