#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#![feature(extern_types)]
#[rustc_has_incoherent_inherent_impls]
pub trait T{}
#[rustc_has_incoherent_inherent_impls]
pub struct S;
#[rustc_has_incoherent_inherent_impls]
pub enum E{Unit}
#[rustc_has_incoherent_inherent_impls]
pub union U{pub value:u8}
unsafe extern "C"{#[rustc_has_incoherent_inherent_impls]pub type Opaque;}
