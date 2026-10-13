fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(align(64))]#[derive(Clone,Copy)]struct Z;fn f(s:&[Z]){let _=&s[2];}fn main(){f(&[Z;3]);}
