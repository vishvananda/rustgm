#![feature(const_trait_impl)]
pub const trait Base{fn read()->u8;}
pub const trait Maker{type Output:[const]Base;}
impl<T>Base for T{fn read()->u8{41u8}}
pub const fn read<M:Maker>()->u8{<M::Output as Base>::read()}
fn main(){}
