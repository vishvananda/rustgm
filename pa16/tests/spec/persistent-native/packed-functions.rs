fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u64)->u64{x+1}#[repr(C,packed)]struct P{a:u8,f:fn(u64)->u64,b:u16}static A:P=P{a:9,f:inc,b:513};fn main(){check((A.f)(41)==42 && A.a==9 && A.b==513);}
