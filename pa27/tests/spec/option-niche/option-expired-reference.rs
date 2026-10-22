union U{o:Option<&'static u32>,p:*const u32}const N:Option<&u32>=unsafe{let p={let v=37;&raw const v};(U{p}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
