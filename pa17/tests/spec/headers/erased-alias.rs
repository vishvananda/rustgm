struct S;type A<'a>=S;impl<'a> Drop for A<'a>{fn drop(&mut self){}}fn main(){}
