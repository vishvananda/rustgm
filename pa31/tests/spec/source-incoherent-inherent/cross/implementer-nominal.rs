#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
pub use authority::Token;
impl Token{#[rustc_allow_incoherent_impl]pub fn read(&self)->u8{self.0}}
pub fn anchor(){}
