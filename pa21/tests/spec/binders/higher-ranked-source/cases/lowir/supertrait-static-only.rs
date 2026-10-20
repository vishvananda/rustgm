trait A<'a>{}trait B:for<'a>A<'a>{}impl A<'static>for u8{}impl B for u8{}fn main(){}
