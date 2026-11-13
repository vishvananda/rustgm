trait Value{const VALUE:usize;}impl Value for u8{const VALUE:usize=7;}
const fn generic<T:Value>()->usize{const{let p=&T::VALUE;*p}}const ACTUAL:usize=generic::<u8>();
#[unsafe(no_mangle)]pub extern "C" fn observe()->usize{ACTUAL}
