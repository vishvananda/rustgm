fn f<const N:usize>()->impl Copy{7u8}fn same<T>(_:T,_:T){}fn main(){same(f::<3>(),f::<4>());}
