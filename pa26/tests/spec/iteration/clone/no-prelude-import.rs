#![no_implicit_prelude]
extern crate core;use core::clone::Clone as C;fn check(b:bool){if !b{let z=0;let _=1/z;}}struct V(u8);fn main(){let a=[V(8)];let mut i=a.iter().clone();match core::iter::Iterator::next(&mut i){core::option::Option::Some(x)=>check(x.0==8),core::option::Option::None=>check(false)}}
