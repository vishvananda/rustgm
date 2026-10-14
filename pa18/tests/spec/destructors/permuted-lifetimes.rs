fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,'b,T>(&'a mut u8,&'b u8,T);impl<'x,'y,T> Drop for D<'y,'x,T>{fn drop(&mut self){*self.0+=*self.1;}}fn main(){let mut n=0;let k=17;{let _d=D(&mut n,&k,9u16);}check(n==17);}
