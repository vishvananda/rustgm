fn f(){let _=need(make());}fn need<T:B>(_:T){}trait A{}trait B{}impl<T:Send>B for T{}impl A for u8{}fn make()->impl A{7u8}
fn main(){}
