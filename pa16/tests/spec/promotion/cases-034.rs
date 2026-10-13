fn g(){}struct S{a:u8}fn f()->&'static S{&S{a:{g();1}}}
fn main() {}
