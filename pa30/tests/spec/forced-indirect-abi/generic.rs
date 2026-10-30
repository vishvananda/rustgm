#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S<T>(T);pub fn observe()->u64{S(7_u64).0}
