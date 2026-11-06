#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
struct S<'a>{first:&'a mut u8,second:String}impl<'a>Reborrow for S<'a>{}
