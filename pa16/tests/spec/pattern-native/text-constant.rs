fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const A:&str="héllo";const R:&&str=&&"🦀";fn f(s:&str)->u8{match s{A=>7,_=>0}}fn g(s:&&str)->u8{match s{R=>9,_=>0}}fn main(){check(f(A)==7 && f("hello")==0 && g(&"🦀")==9 && g(&"héllo")==0);}
