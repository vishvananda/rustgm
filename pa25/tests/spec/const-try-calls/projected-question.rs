#![feature(try_blocks,const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
use core::ops::{Try,FromResidual,Residual,ControlFlow};
pub const trait Maker{type Output:[const]Try<Output=u8,Residual:[const]Destruct>;}
pub const fn apply<M:[const]Maker>(v:M::Output)->M::Output where <M::Output as Try>::Residual:[const]Residual<u8,TryType=M::Output>+[const]Destruct{try{v?}}
fn main(){}
