fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:usize,const I:usize>()->&'static u8{&[3;N][I]}fn main(){let _=f::<0,0>();}
