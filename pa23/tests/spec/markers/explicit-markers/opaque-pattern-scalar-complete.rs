fn main(){need(make());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}const C:u8=1;fn make()->impl A{match 1u8{C=>7u8,_=>9}}
