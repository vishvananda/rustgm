fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}
fn f(n:&mut u8){let _d=D(n);return;}fn main(){let mut n=0;f(&mut n);check(n==1);}
