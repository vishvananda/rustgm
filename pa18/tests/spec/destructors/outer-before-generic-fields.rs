fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T>(&'a mut u16,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0=*self.0*10+1;}}struct W<'a,T>(D<'a,T>);impl<T> Drop for W<'_,T>{fn drop(&mut self){*self.0.0=*self.0.0*10+2;}}fn main(){let mut n=0;{let _w=W(D(&mut n,7u8));}check(n==21);}
