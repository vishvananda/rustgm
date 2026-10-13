fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[fn();0]=[];fn f(C:[fn();0])->u8{7}fn main(){check(f([])==7);}