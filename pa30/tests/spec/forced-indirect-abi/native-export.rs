#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);#[unsafe(no_mangle)]pub extern "C" fn read_value(v:S)->u64{v.0}
