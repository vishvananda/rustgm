fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->&'static u8{&(N/2)}fn main(){check(*f::<255>()==127);}
