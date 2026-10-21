#[deny(no_mangle_generic_items)]#[unsafe(export_name="ffi")]extern "C" fn f<T>(x:T)->T{x}fn main(){}
