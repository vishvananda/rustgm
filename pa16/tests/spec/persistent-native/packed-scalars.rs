fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed)]struct P{a:u8,b:u128,c:u16}static A:P=P{a:9,b:340282366920938463463374607431768211455,c:513};fn main(){check(A.a==9 && A.b==340282366920938463463374607431768211455 && A.c==513);}
