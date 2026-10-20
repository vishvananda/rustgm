fn f(){need(make());}fn need<T:Send>(_:T){}fn make()->impl A{7u8}trait A{}impl A for u8{}
fn main(){}
