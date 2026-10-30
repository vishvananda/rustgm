trait A{}trait B{}impl A for u8{}impl<T:A>B for T{}impl B for u8{}
