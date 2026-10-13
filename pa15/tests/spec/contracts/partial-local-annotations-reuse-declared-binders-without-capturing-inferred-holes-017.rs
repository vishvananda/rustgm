fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{let p:fn(&_)->&_=id;let _=p(y);p(x)}
fn main() {}
