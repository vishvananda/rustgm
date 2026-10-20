trait A{}impl A for u8{}trait B<'a>{}impl<'a,T:Send>B<'a> for T{}fn outer()->impl for<'a>B<'a>{inner()}fn inner()->impl A{7u8}
fn main(){}
