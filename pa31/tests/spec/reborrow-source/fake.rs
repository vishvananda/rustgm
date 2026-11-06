#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
mod unrelated{pub trait Reborrow{}}
struct Exclusive<'a>{value:&'a mut u8}impl unrelated::Reborrow for Exclusive<'_>{}
fn take(_:Exclusive<'_>){}
pub fn bad(value:&mut u8){let value=Exclusive{value};take(value);take(value);}
