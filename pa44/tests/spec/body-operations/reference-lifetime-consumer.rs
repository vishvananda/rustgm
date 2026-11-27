#![feature(async_drop)]
#![no_std]
use minicore::{Future,cleanup};
fn require_static<F:Future<Output=()>+'static>(_:F){}
fn consume(x:&mut u8){require_static(unsafe{cleanup(x)});}
