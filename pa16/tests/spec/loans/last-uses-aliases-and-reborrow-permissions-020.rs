fn f(){let mut x=0;let mut y=0;let mut p=&mut x;let q=&mut *p;p=&mut y;*q=1;*p=2;}
fn main() {}
