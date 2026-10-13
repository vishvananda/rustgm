struct S<'a>(&'a u8);fn f<'a>(x:&'a u8){let y:(S<'a>,_)= (S(x),1u8);}
fn main() {}
