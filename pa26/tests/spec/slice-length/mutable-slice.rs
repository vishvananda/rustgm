fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[1u8,2,3];let a:&mut[u8]=&mut a;check(a.len()==3);a[0]=7;check(a[0]==7);}
