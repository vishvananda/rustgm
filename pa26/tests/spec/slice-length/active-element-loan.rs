fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[1u8,2,3];let r=&mut a[0];let _=a.len();*r=7;}
