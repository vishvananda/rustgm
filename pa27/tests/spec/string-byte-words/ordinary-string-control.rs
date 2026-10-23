const S:&str="h\u{e9}";const N:u8=unsafe{*(S as *const str as *const u8)};fn main(){if N!=104{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
