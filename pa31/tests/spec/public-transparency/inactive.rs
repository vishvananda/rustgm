#![feature(rustc_attrs)]
#[cfg(any())]#[rustc_pub_transparent(bogus)]fn removed(){}
pub fn observed()->u64{7}
