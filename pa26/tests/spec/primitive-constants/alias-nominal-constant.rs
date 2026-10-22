fn check(b:bool){if !b{let z=0;let _=1/z;}}struct S;impl S{const MAX:u8=7;}type A=S;fn main(){check(A::MAX==7);}
