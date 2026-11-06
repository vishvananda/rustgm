#![feature(inherent_associated_types)]
mod m{pub struct S;impl S{pub type A=u8;}}fn f(x:m::S::A)->u8{x}
