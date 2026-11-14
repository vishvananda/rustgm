#![feature(const_trait_impl)]
pub const trait Base{fn read(&self)->u8;}
pub const trait Middle:[const]Base{}
pub const trait Derived:[const]Middle{fn through(&self)->u8{Base::read(self)}}
fn main(){}
