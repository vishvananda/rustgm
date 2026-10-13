fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;fn f(x:u8)->u8{let n@N=x else{return 0;};n}fn main(){check(f(7)==7&&f(0)==0);}