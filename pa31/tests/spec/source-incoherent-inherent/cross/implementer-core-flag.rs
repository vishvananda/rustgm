#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#![rustc_coherence_is_core]
impl authority::Token{#[rustc_allow_incoherent_impl]pub fn read(&self)->u8{self.0}}
