static S:[u8;8]=[0;8];union U{o:Option<&'static u32>,p:*const u32}const N:Option<&u32>=unsafe{(U{p:(&raw const S[1])as *const u32}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
