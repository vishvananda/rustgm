#![feature(rustc_attrs,allow_internal_unstable)]
#[allow_internal_unstable(with_negative_coherence)]macro_rules! m{()=>{#[rustc_strict_coherence]trait A{}}}m!();fn main(){}
