fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[u8;2]=[2,7];fn f(x:[u8;2])->u8{let n@C=x else{return 0;};n[1]}fn main(){check(f([2,7])==7&&f([7,2])==0);}