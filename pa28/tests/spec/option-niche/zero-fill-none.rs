static S:u32=37;const fn run()->bool{let mut v=Some(&S);unsafe{core::ptr::write_bytes(&raw mut v,0,1)}match v{None=>true,Some(_)=>false}}const N:bool=run();fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
