static S:u32=37;union U{o:Option<&'static u32>,p:*const u32}const N:Option<&u32>=unsafe{(U{p:(&raw const S).add(1)}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
