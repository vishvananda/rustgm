#![feature(try_blocks,try_trait_v2,try_trait_v2_residual)]
use std::ops::{ControlFlow,FromResidual,Residual,Try};
enum Outcome<T>{Value(T),Error(u8)}
enum Short{Error(u8)}
impl<T>Residual<T> for Short{type TryType=Outcome<T>;}
impl<T>FromResidual<Short> for Outcome<T>{fn from_residual(_:Short)->Self{Outcome::Error(23)}}
impl<T>Try for Outcome<T>{type Output=T;type Residual=Short;fn from_output(value:T)->Self{Outcome::Value(value)}fn branch(self)->ControlFlow<Short,T>{match self{Outcome::Value(value)=>ControlFlow::Continue(value),Outcome::Error(error)=>ControlFlow::Break(Short::Error(error))}}}
struct Weird(u8);enum WeirdResidual{Error(u8)}
impl Residual<u8>for WeirdResidual{type TryType=Weird;}
impl FromResidual<WeirdResidual>for Weird{fn from_residual(_:WeirdResidual)->Self{Weird(47)}}
impl Try for Weird{type Output=u8;type Residual=WeirdResidual;fn from_output(_:u8)->Self{Weird(43)}fn branch(self)->ControlFlow<WeirdResidual,u8>{ControlFlow::Continue(self.0)}}
unsafe extern "C"{fn record(tag:u8,value:u8);}
fn read(value:Outcome<u8>)->u8{match value{Outcome::Value(value)=>value,Outcome::Error(error)=>error}}
#[unsafe(no_mangle)]pub extern "C" fn code(value:u8)->u8{
 let run=move||->Outcome<u8>{try{value}};let value=read(run());unsafe{record(1,value)};
 let value=41u8;let run=move||->Weird{try{value}};let value=run().0;unsafe{record(2,value)};
 let value=Outcome::<u8>::Error(9);let run=move||->Outcome<u8>{try{value?}};let value=read(run());unsafe{record(3,value)};value
}
