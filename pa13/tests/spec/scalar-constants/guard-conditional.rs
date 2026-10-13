fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;fn f(x:u8)->u8{match x{n if let N=n=>1,_=>2}}fn main(){check(f(7)==1&&f(0)==2);}