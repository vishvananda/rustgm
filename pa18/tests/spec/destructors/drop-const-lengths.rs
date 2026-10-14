fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a,const N:usize>(&'a mut u8,[u8;N]);impl<const N:usize> Drop for D<'_,N>{fn drop(&mut self){*self.0+=1;}}fn main(){let mut n=0;{let _a=D(&mut n,[1;0]);}{let _b=D(&mut n,[2;4]);}check(n==2);}
