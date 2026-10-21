fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=7;let r=&mut n;let f=move||r;let q=f();*q=9u8;check(n==9);}
