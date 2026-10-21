fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[3u8,9];let p:*const [u8]=&a;unsafe{check((*p)[1]==9);}}