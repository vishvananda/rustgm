fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn consume(_:D<'_>){}
fn main(){let mut a=0u8;let mut b=0u8;let d1=D(&mut a);let d2=D(&mut b);let f=move||{consume(d1);let _=&d2;};f();check(a==1&&b==1);}
