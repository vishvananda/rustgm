union U{o:Option<&'static u32>,half:u32}const N:Option<&u32>=unsafe{(U{half:0}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
