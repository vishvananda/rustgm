fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W;impl W{const MAX:u8=9;}type usize=W;fn main(){check(usize::MAX==9);}
