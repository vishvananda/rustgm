#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);#[repr(transparent)]struct W(S);#[unsafe(no_mangle)]pub extern "C" fn read_value(v:W)->u64{v.0.0}
