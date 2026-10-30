#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($a:meta)=>{#[$a]fn f<const N:usize>()->usize{N}}}
make!(rustc_legacy_const_generics(0));pub fn observed()->usize{f::<7>()}
