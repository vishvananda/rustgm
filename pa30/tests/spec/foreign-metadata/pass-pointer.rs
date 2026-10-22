fn main(){let f:extern "C" fn(u32,u32)->u32=one::add;let g:unsafe extern "C" fn(u32,u32)->u32=one::add;check(f(20,22)==42);check(unsafe{g(18,24)}==42);}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(n:u8)->u8{1/n}

