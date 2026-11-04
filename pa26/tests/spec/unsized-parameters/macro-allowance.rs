#![feature(allow_internal_unstable)]
#[allow_internal_unstable(unsized_fn_params)]macro_rules! declare{()=>{fn take(_x:str){}}}declare!();fn main(){}
