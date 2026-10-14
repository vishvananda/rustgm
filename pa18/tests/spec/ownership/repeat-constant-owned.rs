struct D;impl Drop for D{fn drop(&mut self){}}const VALUE:D=D;fn f<const N:usize>()->[D;N]{[VALUE;N]}
fn main(){}
