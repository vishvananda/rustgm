trait A{}impl A for u8{}trait B:Send{}impl<T:Send>B for T{}fn outer()->impl B{inner()}fn inner()->impl A{7u8}
fn main(){}
