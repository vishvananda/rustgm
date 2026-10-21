#[deny(no_mangle_generic_items)]#[unsafe(no_mangle)]extern "C" fn ffi<const N:usize>(x:i32)->i32{x}fn main(){}
