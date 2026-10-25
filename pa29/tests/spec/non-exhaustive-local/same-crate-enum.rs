#![allow(dead_code,unused_variables)]
#[non_exhaustive]pub enum E{A,B(u8)}
fn probe(v:E)->u8{match v{E::A=>0u8,E::B(x)=>x}}
