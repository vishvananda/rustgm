union U{o:Option<&'static u32>,bits:u64}const N:Option<&u32>=unsafe{(U{bits:1}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
