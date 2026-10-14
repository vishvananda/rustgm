fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,'b,T>(&'a mut u8,&'b u8,T)where 'a:'b;impl<'a,'b,'u,T> Drop for D<'a,'b,T> where 'a:'u,'u:'b{fn drop(&mut self){*self.0+=*self.1;}}fn main(){let mut n=0;let k=3;{let _d=D(&mut n,&k,7u8);}check(n==3);}
