static S:bool=true;#[derive(Copy,Clone)]enum E{Only(&'static u32)}union U{e:E,p:*const u8}const N:E=unsafe{(U{p:(&raw const S)as *const u8}).e};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
