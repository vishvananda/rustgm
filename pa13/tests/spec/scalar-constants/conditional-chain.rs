fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;fn f(x:u8)->u8{if let n@N=x&&n>0{n}else{0}}fn main(){check(f(7)==7&&f(0)==0);}