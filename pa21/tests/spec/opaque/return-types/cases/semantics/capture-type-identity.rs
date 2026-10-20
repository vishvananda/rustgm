fn f<T>(_:T)->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f(7u8),f(false));}
