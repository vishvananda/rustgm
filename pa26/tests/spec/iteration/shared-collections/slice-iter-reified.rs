fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let f:for<'a>fn(&'a[u8])->core::slice::Iter<'a,u8>=<[u8]>::iter;let a=[2u8,3];let mut n=0u8;for x in f(&a){n+=*x;}check(n==5);}
