fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Clone,Copy)]struct S<T>(T);fn main(){let a=S(9u8);let b=a;check(a.0+b.0==18);}
