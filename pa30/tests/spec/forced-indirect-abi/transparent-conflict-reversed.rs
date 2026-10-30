#![feature(rustc_attrs)]
#[repr(transparent)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);
