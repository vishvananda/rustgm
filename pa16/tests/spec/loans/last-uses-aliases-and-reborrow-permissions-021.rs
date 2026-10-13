fn f(){let x=0;let mut p=&x;let q=&*p;p=&1;let y=*q;}
fn main() {}
