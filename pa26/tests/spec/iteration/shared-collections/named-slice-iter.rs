fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[1u8,2];let i:core::slice::Iter<'_,u8>=a.iter();let mut n=0u8;for x in i{n+=*x;}check(n==3);}
