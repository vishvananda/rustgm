fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}
fn main(){let mut n=0;let mut i=0;while i<3{let _d=D(&mut n);i+=1;if i<3{continue;}break;}check(n==3);}
