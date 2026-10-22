fn check(b:bool){if !b{let z=0;let _=1/z;}}trait Clone{fn clone(&self)->u8;}struct X;impl Clone for X{fn clone(&self)->u8{9}}fn main(){check(X.clone()==9);}
