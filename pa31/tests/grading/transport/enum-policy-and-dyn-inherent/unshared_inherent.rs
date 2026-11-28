#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

pub fn rejected(marker: &dyn base::Marker) -> u8 {
    marker.shared_tag()
}
