#![feature(rustc_attrs)]
mod a{#[rustc_no_mir_inline]pub fn f()->u32{7}}mod b{#[rustc_no_mir_inline]pub fn f()->u32{9}}
