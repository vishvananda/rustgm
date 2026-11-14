#![feature(const_trait_impl)]
pub const trait Base{fn read(&self)->u8;}
pub const trait Derived:[const]Base{
fn through_0(&self)->u8{Base::read(self)}
fn through_1(&self)->u8{Base::read(self)}
fn through_2(&self)->u8{Base::read(self)}
fn through_3(&self)->u8{Base::read(self)}
fn through_4(&self)->u8{Base::read(self)}
fn through_5(&self)->u8{Base::read(self)}
fn through_6(&self)->u8{Base::read(self)}
fn through_7(&self)->u8{Base::read(self)}
}
fn main(){}
