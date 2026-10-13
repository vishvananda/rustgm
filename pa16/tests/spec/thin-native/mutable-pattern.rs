fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&mut (u8,u8)){let (a,b)=x;*a+=*b;}fn main(){let mut p=(3,4);f(&mut p);check(p.0==7 && p.1==4);}
