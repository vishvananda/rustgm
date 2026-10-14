fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}
fn run(flag:bool)->u8{let mut n=0;{let d:D<'_>;if flag{d=D(&mut n);}}n}fn main(){check(run(false)==0);check(run(true)==1);}
