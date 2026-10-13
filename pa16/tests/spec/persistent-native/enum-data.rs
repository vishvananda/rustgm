fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Clone,Copy)]#[repr(C,u8)]enum E{A(u64,u8),B{n:u128}}static A:E=E::A(42,7);const B:E=E::B{n:340282366920938463463374607431768211455};fn main(){let a=match A{E::A(n,b)=>n+b as u64,_=>0};let b=match B{E::B{n}=>n,_=>0};check(a==49 && b==340282366920938463463374607431768211455);}
