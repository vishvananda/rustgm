fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[1u8,2];let mut b=[1u8,2];let s:&mut[u8]=&mut b;check(a==s);check(s==a);s[0]=3;check(b[0]==3);}
