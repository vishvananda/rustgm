#![feature(rustc_attrs)]
mod a{#[rustc_no_writable]pub fn f()->u32{7}}mod b{#[rustc_no_writable]pub fn f()->u32{9}}
