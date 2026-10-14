use ::c::ops::Drop as D;extern crate core as c;impl D for S{fn drop(&mut self){}}struct S;fn main(){let _s=S;}
