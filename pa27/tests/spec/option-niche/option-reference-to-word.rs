static S:u32=37;union U{o:Option<&'static u32>,p:*const u32}const N:*const u32=unsafe{(U{o:Some(&S)}).p};fn main(){check(unsafe{*N}==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
