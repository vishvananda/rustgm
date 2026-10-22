fn check(b:bool){if !b{let z=0;let _=1/z;}}struct X;trait Local{fn clone(&self)->u8;}impl Local for X{fn clone(&self)->u8{9}}fn main(){let a=[1u8];let _i=a.iter();check(X.clone()==9);}
