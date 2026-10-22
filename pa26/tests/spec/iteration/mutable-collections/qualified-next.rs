fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut a=[1u8];let mut i=<[u8]>::iter_mut(&mut a);match core::iter::Iterator::next(&mut i){Some(x)=>*x=9,None=>check(false)}check(a[0]==9);}
