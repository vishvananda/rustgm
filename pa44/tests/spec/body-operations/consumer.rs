#![feature(no_core,async_drop)]
#![no_core]
use minicore::{Future,cleanup};
fn require<F:Future<Output=()>>(_:F){}
fn consume(x:&mut u8,y:&mut u16){require(unsafe{cleanup(x)});require(unsafe{cleanup(y)});}
