#![feature(optimize_attribute)]
#[optimize(size)]fn a(x:u32)->u32{x+1}
#[optimize(speed)]fn b(x:u32)->u32{x*2}
#[optimize(none)]#[inline(never)]fn c(x:u32)->u32{x-3}
fn main(){println!("{}",c(b(a(6))));}
