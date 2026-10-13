fn f(){let mut x=0;let mut y=0;let mut p=&mut x;*p=1;p=&mut y;x=2;*p=3;}
fn main() {}
