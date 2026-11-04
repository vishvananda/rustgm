#![feature(allow_internal_unstable)]
trait A{}#[allow_internal_unstable(negative_impls)]
macro_rules! m{()=>{impl !A for u32{}}}m!();fn main(){}
