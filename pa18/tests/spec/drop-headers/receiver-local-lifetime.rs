struct S<'a,T>(&'a T);impl<'a,T> Drop for S<'a,T>{fn drop<'r>(&'r mut self){let r=self.0;}}
fn main(){}
