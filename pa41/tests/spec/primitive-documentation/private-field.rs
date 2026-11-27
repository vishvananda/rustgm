#![feature(rustc_attrs)]
#[rustc_doc_primitive="u64"]mod docs{pub struct S(u64);}pub fn f(){let _=docs::S(7);}
