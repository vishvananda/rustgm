fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn main(){let mut x=S(7);let mut n=0;while let C=x{n+=1;x=S(0);}check(n==1);}