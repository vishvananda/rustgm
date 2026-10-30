#![feature(rustc_attrs)]
const _:()={#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct Type(usize);const extern "C" fn c(_:Type){}c(Type(0))};
