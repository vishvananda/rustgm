#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);fn take(x:S)->u64{x.0}pub fn observe()->u64{take(S(7))}
