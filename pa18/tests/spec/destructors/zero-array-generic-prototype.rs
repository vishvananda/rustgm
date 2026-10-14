fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;{let _a=id([D(&mut n,7u8);0]);}check(n==1);}
