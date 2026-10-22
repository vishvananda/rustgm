enum E<T>{A(T),Empty}const fn read<T:Copy>(p:&T)->T{*p}const N:u32={let e=E::A(||37u32);match &e{E::A(f)=>{let v=read(f);let _=v;37},E::Empty=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
