#![feature(rustc_attrs)]
#[rustc_doc_primitive="bool"]mod docs{pub fn value()->u64{7}}const X:u64=docs::value();
