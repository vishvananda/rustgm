trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}type P<'a,'b>=H<'a,'b,dyn A+'static>;fn main(){}
