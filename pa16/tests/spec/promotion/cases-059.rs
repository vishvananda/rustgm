fn g(){}const C:fn()=g as fn();fn f()->&'static fn(){&C}
fn main() {}
