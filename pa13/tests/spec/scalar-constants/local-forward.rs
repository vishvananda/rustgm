fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:u8)->u8{let n=match x{N=>7,_=>9};const N:u8=3;n}fn main(){check(f(3)==7&&f(2)==9);}