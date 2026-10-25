#![allow(dead_code,unused_variables,internal_features)]
#![feature(rustc_attrs)]
#[rustc_on_unimplemented(message="NEEDS {Self}")]trait Need{}impl Need for u32{}fn require<T:Need>(){}fn probe(){require::<u32>();}
fn main(){}
