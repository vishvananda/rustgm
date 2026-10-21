fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let _=<u8 as PartialOrd<u16>>::partial_cmp(&1,&2);}
