struct S<T:Copy>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self){let x=self.0;let _=x;}}
fn main(){}
