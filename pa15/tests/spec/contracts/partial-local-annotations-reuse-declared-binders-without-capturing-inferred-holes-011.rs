fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{let p:fn(&_) -> &_=id;p(x)}
fn main() {}
