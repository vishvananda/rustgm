union U{a:u32,b:u8}const X:(u32,u8)={let mut u=U{a:0x12345678};let a=&raw mut u.a;let b=&raw mut u.b;unsafe{*b=0xab;let v=*a;*a=0x31415926;(v,*b)}};
fn main(){check(X.0==0x123456ab);check(X.1==0x26);}
