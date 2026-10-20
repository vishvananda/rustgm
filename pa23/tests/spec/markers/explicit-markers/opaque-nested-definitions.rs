fn main(){need(make0());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}fn make0()->impl A{need(make1());7u8}fn make1()->impl A{need(make2());7u8}fn make2()->impl A{7u8}
