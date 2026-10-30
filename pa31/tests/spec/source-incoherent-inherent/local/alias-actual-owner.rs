#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_has_incoherent_inherent_impls]
pub struct S;
type Alias=S;
impl Alias{pub fn value(&self)->u8{7}}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{S.value()}
