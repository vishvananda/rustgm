trait A{}trait B<'a,T:?Sized+'a>{}fn f<'a>(x:&'a dyn B<'a,dyn A+'a>)->&'a dyn B<'a,dyn A>{x}fn main(){}
