fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=1u8;let a=&mut x;let shared=&a;let _=<&mut u8 as PartialEq>::eq(&a,&a);*a=2;check(**shared==2);}
