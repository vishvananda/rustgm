#![feature(allow_internal_unstable)]
trait A{}#[allow_internal_unstable(negative_impls)]
macro_rules! m{($i:item)=>{$i}}m!(impl !A for u32{});fn main(){}
