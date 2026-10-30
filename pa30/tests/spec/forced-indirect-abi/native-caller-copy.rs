#![feature(rustc_attrs)]
#[derive(Clone,Copy)]#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);unsafe extern "C"{fn change(v:S)->u64;}#[unsafe(no_mangle)]pub extern "C" fn native_observed()->u64{let s=S(7);let changed=unsafe{change(s)};changed+s.0}
