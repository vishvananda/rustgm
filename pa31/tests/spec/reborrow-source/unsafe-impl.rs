#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
struct Exclusive<'a>{value:&'a mut u8}
unsafe impl<'a>Reborrow for Exclusive<'a>{}
struct Shared<'a>{value:&'a u8}
static mut CLONES:u8=0;
impl Clone for Shared<'_>{fn clone(&self)->Self{unsafe{CLONES=99;}Self{value:self.value}}}
impl Copy for Shared<'_>{}
impl<'a>CoerceShared<Shared<'a>>for Exclusive<'a>{}
fn write(value:Exclusive<'_>,next:u8){*value.value=next;}
fn read(value:Shared<'_>)->u8{*value.value}
