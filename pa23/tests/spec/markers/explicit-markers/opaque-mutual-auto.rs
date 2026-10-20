trait A{}impl A for u8{}fn need<T:Send>(_:T){}fn a()->impl A{need(b());7u8}fn b()->impl A{need(a());7u8}
fn main(){}
