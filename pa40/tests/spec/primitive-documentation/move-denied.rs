#![feature(rustc_attrs)]
#[rustc_doc_primitive="u64"]mod docs{pub struct Owned(pub u64);}pub fn f()->u64{let v=docs::Owned(7);let a=v;let b=v;a.0+b.0}
