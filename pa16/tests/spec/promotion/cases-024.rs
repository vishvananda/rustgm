fn g(){}fn f()->&'static fn(){&(g as fn())}
fn main() {}
