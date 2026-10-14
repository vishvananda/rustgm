struct D;impl Drop for D{fn drop(&mut self){}}fn f<T>()->&'static D{&D}
fn main(){}
