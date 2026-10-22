fn check(b:bool){if !b{let z=0;let _=1/z;}}struct X;impl X{fn clone(&self)->u8{9}}fn main(){let a=[1u8];let _i=a.iter();check(X.clone()==9);}
