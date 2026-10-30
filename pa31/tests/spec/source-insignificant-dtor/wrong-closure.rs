#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#![feature(stmt_expr_attributes)]
fn f(){let _=#[rustc_insignificant_dtor]||7u8;}
