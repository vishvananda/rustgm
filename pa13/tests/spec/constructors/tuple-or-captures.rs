fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:(u8,u8))->u8{match x{(0,n)|(n,0)=>n,_=>99}}fn main(){check(f((0,5))==5&&f((7,0))==7&&f((8,9))==99&&f((0,0))==0);}