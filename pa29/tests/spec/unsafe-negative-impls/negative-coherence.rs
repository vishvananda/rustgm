#![feature(no_core,with_negative_coherence)]
#![no_core]
extern crate base;
trait Pick { fn pick(&self) -> u8; }
impl<T: base::DerefMutLike> Pick for T { fn pick(&self) -> u8 { 42 } }
impl<T: ?base::Sized> Pick for &T { fn pick(&self) -> u8 { 7 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x: u8 = 0; match Pick::pick(&x) { 42 => 42, _ => 7 } }
