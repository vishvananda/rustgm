fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
extern crate first;fn main(){let p=&raw mut first::MUT;unsafe{*p=47;check(first::MUT==47);}}