#![feature(const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
pub struct Borrowed<'a,F>{pub f:&'a mut F}
pub const fn apply0<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply1<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply2<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply3<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply4<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply5<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply6<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
pub const fn apply7<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,v:u8)->u8{(x.f)(v)}
fn main(){}
