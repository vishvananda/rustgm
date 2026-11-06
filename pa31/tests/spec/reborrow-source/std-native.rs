#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
struct Exclusive<'a>{value:&'a mut u8}
impl<'a>Reborrow for Exclusive<'a>{}
struct Shared<'a>{value:&'a u8}
static mut CLONES:u8=0;
impl Clone for Shared<'_>{fn clone(&self)->Self{unsafe{CLONES=99;}Self{value:self.value}}}
impl Copy for Shared<'_>{}
impl<'a>CoerceShared<Shared<'a>>for Exclusive<'a>{}
fn write(value:Exclusive<'_>,next:u8){*value.value=next;}
fn read(value:Shared<'_>)->u8{*value.value}
#[unsafe(no_mangle)]pub extern "C" fn exclusive(value:&mut u8,next:u8)->u8{let value=Exclusive{value};write(value,next);write(value,next);*value.value}
#[unsafe(no_mangle)]pub extern "C" fn shared(value:&mut u8,next:u8)->u8{let value=Exclusive{value};write(value,next);let _first=read(value);read(value)}
#[unsafe(no_mangle)]pub extern "C" fn clone_count()->u8{unsafe{CLONES}}
#[unsafe(no_mangle)]pub extern "C" fn explicit_clone(value:&u8)->u8{*Shared{value}.clone().value}
const fn consume(value:Shared<'_>)->u8{*value.value}
const fn compute()->u8{let mut value=31;consume(Exclusive{value:&mut value})}
const VALUE:u8=compute();
#[unsafe(no_mangle)]pub extern "C" fn constant()->u8{VALUE}
