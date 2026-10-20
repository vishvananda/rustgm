trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy{7u8}}impl A for u16{fn f()->impl Copy{7u8}}fn same<T>(_:T,_:T){}fn main(){same(<u8 as A>::f(),<u16 as A>::f());}
