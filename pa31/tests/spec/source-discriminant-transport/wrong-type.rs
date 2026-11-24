#![feature(no_core, disc_fixture, discriminant_kind)]
#![no_core]
extern crate base;

pub const WRONG: <base::ExternalTag as base::DiscriminantKind>::Discriminant = 3u8;
