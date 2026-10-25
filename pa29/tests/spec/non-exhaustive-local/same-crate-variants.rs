#![allow(dead_code,unused_variables)]
pub enum E{#[non_exhaustive]Named{x:u8},#[non_exhaustive]Tuple(u8),#[non_exhaustive]Unit}
fn probe(v:E)->u8{let first=E::Named{x:7u8};let second=E::Tuple(2u8);let third=E::Unit;match v{E::Named{x}=>x,E::Tuple(x)=>x,E::Unit=>0u8}}
