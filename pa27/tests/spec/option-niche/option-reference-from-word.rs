static S:u32=37;union U{o:Option<&'static u32>,p:&'static u32}const N:Option<&u32>=unsafe{(U{p:&S}).o};fn main(){check(match N{Some(v)=>*v==37,None=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
