#![feature(rustc_attrs)]
#[repr(C,packed)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);
