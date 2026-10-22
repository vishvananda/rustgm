fn main(){let f:unsafe extern "C" fn(u32)->u32=one::guarded;check(unsafe{f(41)}==42);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

