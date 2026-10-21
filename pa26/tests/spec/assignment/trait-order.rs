fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign<u32> for C{fn add_assign(&mut self,r:u32){self.0+=r;}}fn place<'a>(x:&'a mut C,n:&mut u32)->&'a mut C{*n=*n*10+1;x}fn rhs(n:&mut u32)->u32{*n=*n*10+2;4}fn main(){let mut c=C(3);let mut n=0;*place(&mut c,&mut n)+=rhs(&mut n);check(c.0==7&&n==12);}
