#![feature(no_core, disc_fixture, discriminant_kind)]
#![no_core]
extern crate base;

pub struct Local;
impl base::DiscriminantKind for Local { type Discriminant = u8; }
