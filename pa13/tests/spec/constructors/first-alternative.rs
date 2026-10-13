fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:(u8,u8))->u8{match x{(n,_)|(_,n)=>n}}fn main(){check(f((7,9))==7);}