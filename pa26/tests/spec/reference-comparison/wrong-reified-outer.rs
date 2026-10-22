fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let _:fn(&mut &u8,&&u8)->bool=<&u8 as core::cmp::PartialEq>::eq;}