#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
enum Maybe<'a> { Empty, Live(&'a u8) }
static VALUE:u8=9;
const EMPTY:isize=discriminant_value(&Maybe::Empty);
const LIVE:isize=discriminant_value(&Maybe::Live(&VALUE));
fn main(){println!("{} {}",EMPTY,LIVE);}
