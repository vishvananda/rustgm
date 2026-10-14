fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:[E;0]}type A<const N:usize>=[u8;N];fn main(){let x:A<{(&S{e:[]}.e,1).1}>=[7;1];check(x[0]==7);check(E::B as u64==4294967296);}
