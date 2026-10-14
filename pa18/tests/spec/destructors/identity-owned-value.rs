fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn id<T>(x:T)->T{x}fn main(){let mut n=0;{let _d=id(D(&mut n));}check(n==1);}
