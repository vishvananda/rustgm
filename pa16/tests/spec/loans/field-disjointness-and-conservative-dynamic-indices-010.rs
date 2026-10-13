fn f(){let mut x=(0,0);let p=&mut x;let q=&mut p.0;let r=&mut p.1;*q=1;*r=2;}
fn main() {}
