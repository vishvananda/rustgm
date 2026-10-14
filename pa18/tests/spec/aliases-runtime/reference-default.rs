fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<'a,T=&'a u16>=T;fn f<'b>(x:A<'b>)->&'b u16{x}fn main(){let x=517;check(*f(&x)==517);}
