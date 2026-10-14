fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:i8>()->&'static i8{&(-N)}fn main(){let _=f::<{-128}>();}
