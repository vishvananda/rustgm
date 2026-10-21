struct D<'a>(&'a u8);impl<'a> Drop for D<'a>{fn drop(&mut self){let _x=*self.0;}}struct W<T>(T);fn main(){let value;{let n=7;value=W(D(&n));}}
