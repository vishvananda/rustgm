#![feature(try_blocks,const_trait_impl,const_try,const_try_residual,try_trait_v2,try_trait_v2_residual,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
use core::ops::{Try,FromResidual,Residual,ControlFlow};
pub enum Outcome{Value(u8),Error(u8)}
pub enum Short{Error(u8)}
const impl Residual<u8> for Short{type TryType=Outcome;}
const impl FromResidual<Short> for Outcome{fn from_residual(value:Short)->Self{match value{Short::Error(e)=>Outcome::Error(e)}}}
const impl Try for Outcome{type Output=u8;type Residual=Short;
 fn from_output(value:u8)->Self{Outcome::Value(value)}
 fn branch(self)->ControlFlow<Short,u8>{match self{Outcome::Value(v)=>ControlFlow::Continue(v),Outcome::Error(e)=>ControlFlow::Break(Short::Error(e))}}
}
pub const fn make<R:[const]Try<Output=u8>>(v:u8)->R{try{v}}
pub const fn forward<R:[const]Try<Output=u8,Residual:[const]Destruct+[const]Residual<u8,TryType=R>>>(v:R)->R{try{v?}}
pub const VALUE:u8={let a:Outcome=make(41);match a{Outcome::Value(v)=>v,Outcome::Error(e)=>e}};
pub fn observe(value:Outcome)->u8{match value{Outcome::Value(v)=>v,Outcome::Error(e)=>e}}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(v:u8)->u8{observe(forward::<Outcome>(Outcome::Value(v)))}
#[unsafe(no_mangle)]pub extern "C" fn fail()->u8{observe(forward::<Outcome>(Outcome::Error(23)))}
fn main(){}
