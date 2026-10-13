fn g(){}struct S(u8);fn f()->&'static S{&S({g();1})}
fn main() {}
