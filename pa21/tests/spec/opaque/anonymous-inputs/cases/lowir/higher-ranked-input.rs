trait A<'a>{}impl<'a>A<'a>for u8{}fn f(x:impl for<'a>A<'a>){}fn main(){f(7u8);}
