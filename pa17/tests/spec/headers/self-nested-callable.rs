struct S<'a>(&'a u8);fn helper<'a>(_:&S<'a>){}impl<'b> Drop for S<'b>{fn drop(&mut self){let f:fn(&Self)=helper;f(self);}}fn main(){}
