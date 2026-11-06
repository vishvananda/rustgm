#![feature(inherent_associated_types)]
mod m{pub struct S;impl S{type A=u8;}}fn f(_:m::S::A){}
