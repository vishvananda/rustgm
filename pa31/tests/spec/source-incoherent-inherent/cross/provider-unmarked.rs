#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
pub struct Token(pub u8);
pub trait Value{fn value(&self)->u8;}
pub struct Number(pub u8);
impl Value for Number{fn value(&self)->u8{self.0}}
pub use Token as Renamed;
pub type Alias=Token;
