trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn make()->impl A+Send{need(make());7u8}
fn main(){}
