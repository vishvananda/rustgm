#![allow(dead_code,unused_variables)]
#[non_exhaustive]pub enum Never{}
fn probe(v:Never)->u8{match v{}}
