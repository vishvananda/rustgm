fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make<T:Copy>(x:T)->impl Fn()->T{move||x}fn main(){let f=make(7u8);let g=make(900u16);check(f()==7&&g()==900);}
