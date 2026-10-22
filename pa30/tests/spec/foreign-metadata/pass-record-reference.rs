fn main(){let m=two::Marker{value:42};check(two::record_value(&m)==42);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

