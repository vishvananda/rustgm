fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[derive(PartialEq)]struct S([f32;0]);const C:S=S([0.0/0.0;0]);fn f(C:S)->u8{7}fn main(){check(f(S([]))==7);}