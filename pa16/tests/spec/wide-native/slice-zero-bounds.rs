fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Clone,Copy)]struct Z;fn f(s:&[Z]){let _=&s[3];}fn main(){f(&[Z;3]);}
