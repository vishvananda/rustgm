#![feature(async_drop)]
#![no_std]
use base::{Future,cleanup};
fn require<F:Future<Output=u8>>(_:F){}
fn consume(x:&mut u8,y:&mut u16){require(unsafe{cleanup(x)});require(unsafe{cleanup(y)});}
