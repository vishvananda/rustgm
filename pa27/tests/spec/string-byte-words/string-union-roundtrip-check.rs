union U{a:&'static str,b:&'static str}
const S:&str=unsafe{U{a:"h\u{e9}"}.b};
const N:u8=unsafe{*(S as *const str as *const u8)};
fn main(){if N!=104{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
