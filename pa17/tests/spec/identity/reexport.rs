mod m{pub use std::ops::Drop as D;}use m::D;impl D for S{fn drop(&mut self){}}struct S;fn main(){let _s=S;}
