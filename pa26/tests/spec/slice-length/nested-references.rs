fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[1u8,2,3];let r=&mut a;let r=&r;check(r.len()==3);let mut b=[1u8,2];let s:&mut[u8]=&mut b;let s=&s;check(s.len()==2);}
