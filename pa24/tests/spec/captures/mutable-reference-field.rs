fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn main(){let mut b=S(7);let x=&mut b;let mut f=||x.0+=2;f();check(x.0==9);}
