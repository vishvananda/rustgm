#![feature(const_trait_impl)]
pub const trait Base{fn read(&self)->u8;}
pub const trait Derived{fn through(&self)->u8 where Self:[const]Base{Base::read(self)}}
fn main(){}
