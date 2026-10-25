#![allow(dead_code,unused_variables,internal_features)]
#[diagnostic::on_unimplemented(message="STABLE {Self}")]trait Need{}fn require<T:Need>(){}fn probe(){require::<u32>();}
fn main(){}
