fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T,U>(&'a mut u8,T,U);impl<A,B> Drop for D<'_,B,A>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _a=D(&mut n,7u8,1000u16);}check(n==1);}
