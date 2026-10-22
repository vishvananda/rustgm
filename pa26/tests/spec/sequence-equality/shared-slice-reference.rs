fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[1u8,2];let b=[1u8,2];let s:&[u8]=&b;check(a==s);check(s==a);}
