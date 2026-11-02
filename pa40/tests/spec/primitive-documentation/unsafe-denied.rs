#![feature(rustc_attrs)]
#[rustc_doc_primitive="bool"]mod docs{pub unsafe fn value()->u64{7}}pub fn f()->u64{docs::value()}
