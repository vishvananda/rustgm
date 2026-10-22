fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(one::take(one::pair(4))==14);check(two::take(two::pair(8))==18);}
