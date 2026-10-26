#![allow(dead_code,unused_imports)]
use std::mem::{size_of,align_of};
#[repr(transparent)]struct Alignment(usize);
const _: () = assert!(size_of::<Alignment>() == size_of::<usize>());
const _: () = assert!(align_of::<Alignment>() == align_of::<usize>());
fn main(){}
