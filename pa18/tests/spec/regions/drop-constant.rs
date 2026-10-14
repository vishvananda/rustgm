struct D;impl Drop for D{fn drop(&mut self){}}const C:D=D;fn f<T>()->&'static D{&C}
fn main(){}
