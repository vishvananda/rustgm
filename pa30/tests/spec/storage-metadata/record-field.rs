fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;fn main(){let r=&first::RECORD;check(r.n==17);}