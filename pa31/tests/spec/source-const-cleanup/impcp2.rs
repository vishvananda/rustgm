#![feature(const_precise_live_drops)]
#![feature(no_core,const_trait_impl,const_destruct)]#![no_core]
extern crate base;
pub const fn forward<T:[const]base::Destruct,U:[const]base::Destruct>(value:base::Pair<T,U>)->(T,U){base::parts(value)}
