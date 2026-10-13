fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[derive(PartialEq)]struct S{}const C:S=S{..S{}};fn f(C:S)->u8{7}fn main(){let C=S{};check(f(S{})==7);}