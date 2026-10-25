#![allow(dead_code,unused_variables,internal_features)]
mod holder{pub use test as alias;}#[holder::alias]fn gone(){missing!();}
fn main(){}
