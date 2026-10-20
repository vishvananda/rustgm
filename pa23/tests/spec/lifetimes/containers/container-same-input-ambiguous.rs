trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}type P<'a>=H<'a,'a,dyn A>;fn main(){}
