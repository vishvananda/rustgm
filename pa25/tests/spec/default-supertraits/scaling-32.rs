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
fn through_8(&self)->u8{Base::read(self)}
fn through_9(&self)->u8{Base::read(self)}
fn through_10(&self)->u8{Base::read(self)}
fn through_11(&self)->u8{Base::read(self)}
fn through_12(&self)->u8{Base::read(self)}
fn through_13(&self)->u8{Base::read(self)}
fn through_14(&self)->u8{Base::read(self)}
fn through_15(&self)->u8{Base::read(self)}
fn through_16(&self)->u8{Base::read(self)}
fn through_17(&self)->u8{Base::read(self)}
fn through_18(&self)->u8{Base::read(self)}
fn through_19(&self)->u8{Base::read(self)}
fn through_20(&self)->u8{Base::read(self)}
fn through_21(&self)->u8{Base::read(self)}
fn through_22(&self)->u8{Base::read(self)}
fn through_23(&self)->u8{Base::read(self)}
fn through_24(&self)->u8{Base::read(self)}
fn through_25(&self)->u8{Base::read(self)}
fn through_26(&self)->u8{Base::read(self)}
fn through_27(&self)->u8{Base::read(self)}
fn through_28(&self)->u8{Base::read(self)}
fn through_29(&self)->u8{Base::read(self)}
fn through_30(&self)->u8{Base::read(self)}
fn through_31(&self)->u8{Base::read(self)}
}
fn main(){}
