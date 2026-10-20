fn make()->impl Send{7u8}fn need<T:Send>(_:T){}fn f(){need(make());}
fn main(){}
