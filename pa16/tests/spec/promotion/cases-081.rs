fn g(){}fn f()->&'static u8{&([1,2][{g();0}]+1)}
fn main() {}
