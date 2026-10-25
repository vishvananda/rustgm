#![allow(dead_code,unused_variables,internal_features)]
#![feature(rustc_attrs)]
#[rustc_on_unimplemented(on(Args="()",note="EMPTY"),message="NEEDS {Self}")]trait Need<Args>{}fn require<T:Need<()>>(){}fn probe(){require::<u32>();}
fn main(){}
