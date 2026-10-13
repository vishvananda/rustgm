struct S{a:u8,b:u8}fn f(p:&mut S){let a=&mut p.a;let b=&mut p.b;*a=1;*b=2;}
fn main() {}
