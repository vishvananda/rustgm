fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967297}#[repr(packed(8))]struct S{e:E}type A<const N:usize={*(&S{e:E::B}.e) as u8 as usize}>=[u8;N];static V:S=S{e:E::B};fn read(x:A)->u8{x[0]}fn main(){check(read([19])==19);check(V.e as u64==4294967297);}
