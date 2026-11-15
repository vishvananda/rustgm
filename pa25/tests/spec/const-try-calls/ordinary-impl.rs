#![feature(try_blocks,const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
use core::ops::{Try,FromResidual,Residual,ControlFlow};
pub enum Outcome{Value(u8),Error(u8)}
pub enum Short{Error(u8)}
const impl Residual<u8> for Short{type TryType=Outcome;}
const impl FromResidual<Short> for Outcome{fn from_residual(value:Short)->Self{match value{Short::Error(e)=>Outcome::Error(e)}}}
impl Try for Outcome{type Output=u8;type Residual=Short;
 fn from_output(value:u8)->Self{Outcome::Value(value)}
 fn branch(self)->ControlFlow<Short,u8>{match self{Outcome::Value(v)=>ControlFlow::Continue(v),Outcome::Error(e)=>ControlFlow::Break(Short::Error(e))}}
}
pub const fn apply(v:u8)->Outcome{try{v}}
fn main(){}
