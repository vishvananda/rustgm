fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const M:u8=7;trait Pick{const N:u8;}struct S;impl Pick for S{const N:u8={let value=1u8;let r=&value;match 7{M=>*r,_=>2}};}fn f<'a>(_:&'a u8,x:u8)->bool{match x{<S as Pick>::N=>true,_=>false}}fn main(){check(f(&0,1));check(!f(&0,7));}
