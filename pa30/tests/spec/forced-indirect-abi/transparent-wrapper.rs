#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);#[repr(transparent)]struct W(S);pub extern "C" fn take(x:W)->u64{x.0.0}
