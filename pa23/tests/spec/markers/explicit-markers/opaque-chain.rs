trait A{}impl A for u8{}fn a()->impl A{b()}fn b()->impl A{7u8}fn need<T:Send>(_:T){}fn f(){need(a());}
fn main(){}
