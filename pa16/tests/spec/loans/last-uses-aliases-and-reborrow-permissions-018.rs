fn f<'a>(p:&'a mut u8){let q:&'a mut u8=&mut *p;*q=1;*p=2;}
fn main() {}
