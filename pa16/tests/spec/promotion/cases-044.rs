fn g(){}fn f()->&'static fn(){let h=g;&(h as fn())}
fn main() {}
