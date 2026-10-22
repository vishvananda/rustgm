fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let a=[S(3),S(5)];let s=&a as &[S];check(s[1].0==5);check(a[0].0==3);}
