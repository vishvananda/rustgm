fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,const N:usize,const M:usize>(&'a mut u8,[u8;N],[u8;M]);impl<const A:usize,const B:usize> Drop for D<'_,B,A>{fn drop(&mut self){*self.0+=A as u8*10+B as u8;}}fn main(){let mut n=0;{let _d=D(&mut n,[1;2],[1;3]);}check(n==32);}
