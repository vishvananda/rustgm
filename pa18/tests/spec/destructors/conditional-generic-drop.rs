fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}fn f<const X:bool>(n:&mut u8){let d;if X{d=D(n,7u8);}}fn main(){let mut n=0;f::<false>(&mut n);check(n==0);f::<true>(&mut n);check(n==1);}
