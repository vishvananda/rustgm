fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(u32::lt(&1,&2));check(u32::le(&2,&2));check(u32::gt(&3,&2));check(u32::ge(&2,&2));let f:fn(&u8,&u8)->bool=u8::lt;check(f(&1,&2));}
