fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[derive(PartialEq,Eq)]struct S(u8);const C:S=S(7);fn f(x:S)->u8{match x{C=>1,_=>2}}fn main(){check(f(S(7))==1&&f(S(0))==2);}