fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->&'static u16{&(N as u16+1)}fn main(){check(*f::<255>()==256);}
