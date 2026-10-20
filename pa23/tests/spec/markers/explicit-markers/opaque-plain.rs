trait A{}impl A for u8{}fn make()->impl A{7u8}fn need<T:Send>(_:T){}fn f(){need(make());}
fn main(){}
