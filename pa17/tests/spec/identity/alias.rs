use core::ops::Drop as D;impl D for S{fn drop(&mut self){}}struct S;fn main(){let _s=S;}
