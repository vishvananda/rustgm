fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,T>(&'a mut u8,T);impl<'a,'u,T> Drop for D<'a,T> where 'a:'u{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _d=D(&mut n,7u8);}check(n==1);}
