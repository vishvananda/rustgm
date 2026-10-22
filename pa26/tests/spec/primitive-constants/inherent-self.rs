fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W;impl W{fn get()->u32{type A=u32;A::MAX}}fn main(){check(W::get()==4294967295u32);}
