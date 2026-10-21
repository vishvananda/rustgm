fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(PartialEq,Eq)]struct S(u8);fn main(){check(S(7)==S(7));}
