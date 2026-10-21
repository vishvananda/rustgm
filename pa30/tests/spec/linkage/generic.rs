#[deny(no_mangle_generic_items)]#[unsafe(no_mangle)]extern "C" fn ffi<T>(x:T)->T{x}fn main(){}
