fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let x=(S(7),S(9));let y=x.1;let f=||{let _=x.0;};let z=x.0;f();check(y.0==9&&z.0==7);}
