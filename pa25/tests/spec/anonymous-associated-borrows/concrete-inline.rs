trait Value{const VALUE:usize;}impl Value for u8{const VALUE:usize=7;}
#[unsafe(no_mangle)]pub extern "C" fn observe()->usize{const{let p=&<u8 as Value>::VALUE;*p}}
