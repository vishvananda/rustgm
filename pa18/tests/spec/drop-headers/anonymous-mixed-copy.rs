struct S<'a,T:Copy,const N:usize>(&'a T,[u8;N]);impl<const N:usize,T:Copy> Drop for S<'_,T,N>{fn drop(self:&mut Self){let value=*self.0;let again=value;self.1=[0;N];}}
fn main(){}
