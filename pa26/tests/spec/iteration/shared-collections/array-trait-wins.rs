fn check(b:bool){if !b{let z=0;let _=1/z;}}trait X{fn iter(&self)->u8;}impl X for [u8;2]{fn iter(&self)->u8{9}}fn main(){let a=[1u8,2];check(a.iter()==9);}
