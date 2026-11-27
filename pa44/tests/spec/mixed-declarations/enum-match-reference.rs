#![feature(async_drop)]
#![no_std]
pub use core::future::{Future,async_drop_in_place as cleanup};
pub enum Opt<T>{None,Some(T)}
pub fn none<T>()->Opt<T>{Opt::None}
pub fn some<T>(x:T)->Opt<T>{Opt::Some{0:x}}
pub fn or<T>(o:Opt<T>,d:T)->T{match o{Opt::Some(x)=>x,Opt::None=>d}}
