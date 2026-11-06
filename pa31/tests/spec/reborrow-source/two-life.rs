#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
struct S<'a,'b>(&'a mut u8,&'b mut u8);impl<'a,'b>Reborrow for S<'a,'b>{}
