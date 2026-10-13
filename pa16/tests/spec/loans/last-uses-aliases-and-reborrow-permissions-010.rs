fn f(){let mut x=0;let p=&mut x;let q=&x;*p=2;let y=*q;}
fn main() {}
