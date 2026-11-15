#![feature(try_blocks,const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
use core::ops::{Try,FromResidual,Residual,ControlFlow};
pub const trait Maker{type Output:[const]Try<Output=u8>;}
pub const fn apply<M:Maker>(v:u8)->M::Output{try{v}}
fn main(){}
