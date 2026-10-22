union U{o:Option<fn()->u8>,p:usize}const N:Option<fn()->u8>=unsafe{(U{p:1}).o};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
