#![feature(try_blocks,const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
use core::ops::{Try,FromResidual,Residual,ControlFlow};
pub const fn make0<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make1<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make2<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make3<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make4<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make5<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make6<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn make7<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
fn main(){}
