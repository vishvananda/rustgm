trait Value{const VALUE:usize;}impl Value for u8{const VALUE:usize=7;}
fn illegal<T>()->usize{const{let p=&<T as Value>::VALUE;*p}}
