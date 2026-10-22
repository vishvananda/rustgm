#[repr(u8)]enum E<T>{A(T),Empty}const N:u32={let mut x=37;let e=E::A(||{x+=1});match e{E::A(f)=>{let _=f;37},E::Empty=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
