fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[bool;0]=[];fn f(C:[bool;0])->u8{7}fn main(){let C=[];check(f([])==7);}