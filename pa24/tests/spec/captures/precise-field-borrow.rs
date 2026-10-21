fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let x=(S(7),S(9));let f=||x.0.0;let y=x.1;check(f()==7&&y.0==9);}
