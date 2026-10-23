union U{a:*const str,b:*const str}
const P:*const str=unsafe{U{a:"ok" as *const str}.b};
const N:u8=unsafe{*(P as *const u8)};
fn main(){if N!=111{panic!()}}
