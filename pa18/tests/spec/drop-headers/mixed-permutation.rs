struct S<'a,'b,T,U,const N:usize,const M:usize>(&'a T,&'b U,[u8;N],[u16;M]);impl<'x,'y,A,B,const X:usize,const Y:usize> Drop for S<'y,'x,B,A,Y,X> where A:'x,B:'y{fn drop(&mut self){let a=self.0;let b=self.1;let c=&self.2;let d=&self.3;}}
fn main(){}
