fn f(){let _=W(make());}struct W<T:for<'a>B<'a>>(T);trait A{}impl A for u8{}trait B<'a>{}impl<'a:'static,T:Send>B<'a> for T{}fn make()->impl A{7u8}
fn main(){}
