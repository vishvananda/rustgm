#![feature(must_not_suspend)]
#[must_not_suspend]pub struct Guard<'a>(&'a mut u32);impl Drop for Guard<'_>{fn drop(&mut self){*self.0=9;}}pub fn observe()->u32{let mut value=7;{let _guard=Guard(&mut value);}value}
