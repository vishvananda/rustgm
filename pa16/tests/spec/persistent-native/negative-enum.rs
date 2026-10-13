fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Clone,Copy)]#[repr(i8)]enum E{A=-3,B=4}static A:E=E::A;fn main(){check(A as i64 == -3);}
