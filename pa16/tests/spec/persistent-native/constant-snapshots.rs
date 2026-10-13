fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:([u16;2],u8)=([7,11],3);fn main(){let mut a=A;let b=A;a.0[1]=99;check(a.0[1]==99 && b.0[1]==11 && A.1==3);}
