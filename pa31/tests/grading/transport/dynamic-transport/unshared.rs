#![feature(no_core, pointer_metadata_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn require_send<T: ?base::Sized + base::Send>(_: &T) {}
pub fn rejected(visitor: base::Visitor<'_>) { require_send(visitor.inner) }
