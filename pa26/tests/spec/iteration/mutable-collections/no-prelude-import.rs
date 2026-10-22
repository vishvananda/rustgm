#![no_implicit_prelude]
extern crate core;use core::iter::Iterator;fn main(){let mut a=[1u8,2];let mut i=a.iter_mut();match i.next(){core::option::Option::Some(x)=>*x=7,core::option::Option::None=>return}if a[0]!=7{let z=0;let _=1/z;}}
