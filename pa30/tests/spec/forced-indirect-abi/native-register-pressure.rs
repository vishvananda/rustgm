#![feature(rustc_attrs)]
#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);#[unsafe(no_mangle)]pub extern "C" fn pressure(_a:u64,_b:u64,_c:u64,_d:u64,_e:u64,_f:u64,v:S,last:u64)->u64{v.0+last}
