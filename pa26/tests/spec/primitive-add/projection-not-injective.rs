fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn same<A:core::ops::Add<Output=u8>,B:core::ops::Add<Output=u8>>(a:A,b:B)->A{b}fn main(){}
