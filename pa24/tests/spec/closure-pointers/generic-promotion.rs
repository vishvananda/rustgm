fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make<const N:u8>()->&'static fn(u8)->u8{&((|x:u8|x+N) as fn(u8)->u8)}fn main(){check(make::<2>()(7)==9);check(make::<3>()(7)==10);}