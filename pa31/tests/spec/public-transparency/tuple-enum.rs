#![feature(rustc_attrs)]
#[repr(transparent)]#[rustc_pub_transparent]struct Tuple(u16);
#[repr(transparent)]#[rustc_pub_transparent]enum Choice{Value(u8)}
pub fn observed()->u64{let Tuple(x)=Tuple(7);let Choice::Value(y)=Choice::Value(9);(x as u64)+(y as u64)-9}
