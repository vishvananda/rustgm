trait A{}struct H<'a,T:?Sized+'a+'static>{r:&'a T}type P<'a>=H<'a,dyn A>;fn main(){}
