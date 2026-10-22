#[derive(Copy,Clone)]enum E{Only(u32)}union U{e:E,b:[u8;4],half:u16}const N:E=unsafe{(U{half:37}).e};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
