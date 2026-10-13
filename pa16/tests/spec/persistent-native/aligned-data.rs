fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(align(64))]struct A(u64);static S:A=A(42);fn main(){check(S.0==42);}
