fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let a=[S(1),S(2)];check(a.len()==2);check(a[0].0==1);let _=a;}
