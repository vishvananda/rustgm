fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static mut A:[u8;3]=[0;3];const N:usize=unsafe{A.len()};fn main(){let _=N;}
