struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){let _x=self.0;}}
fn main(){}
