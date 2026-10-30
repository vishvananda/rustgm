#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! m{()=>{#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);}}m!();
