fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:[();1000000000000])->u8{match x{[(),..,()]=>7}}fn main(){check(f([();1000000000000])==7);}