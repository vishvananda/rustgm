trait A{fn f<'a,T:Copy>(x:&'a T)->(impl Copy+'a,impl Copy+'a);} impl A for u8{fn f<'a,T:Copy>(x:&'a T)->(impl Copy+'a,impl Copy+'a){(x,x)}} fn main(){let n=7u8;let _=<u8 as A>::f(&n);}
