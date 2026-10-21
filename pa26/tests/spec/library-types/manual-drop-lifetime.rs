struct D<'a>(&'a u8);impl<'a> Drop for D<'a>{fn drop(&mut self){let _x=*self.0;}}fn main(){let value;{let n=7;value=core::mem::ManuallyDrop::new(D(&n));}}
