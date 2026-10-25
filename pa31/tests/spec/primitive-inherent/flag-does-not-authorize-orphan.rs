#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl Clone for u8{fn clone(&self)->Self{*self}}
fn main(){}
