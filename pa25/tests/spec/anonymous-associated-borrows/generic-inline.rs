trait Value{const VALUE:usize;}impl Value for u8{const VALUE:usize=7;}
fn generic<T:Value>()->usize{const{let p=&T::VALUE;*p}}
#[unsafe(no_mangle)]pub extern "C" fn observe()->usize{generic::<u8>()}
