fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}struct W<T>(T,T);fn take<T>(x:T){}fn main(){let mut a=0;let mut b=0;{let w=W(D(&mut a,7u8),D(&mut b,8u8));take(w.0);}check(a==1&&b==1);}
