fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn main(){let x:A<{if true{1}else{(*(&S{e:E::B}.e) as u8 as usize)/(E::A as usize)}}>=[5;1];check(x[0]==5);}
