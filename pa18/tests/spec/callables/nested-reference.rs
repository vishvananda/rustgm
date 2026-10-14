fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:u8>()->&'static &'static u8{&&N}fn main(){check(**f::<7>()==7);check(**f::<9>()==9);}
