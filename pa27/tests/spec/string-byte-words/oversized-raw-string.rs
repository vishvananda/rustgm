static B:u8=65;union U{a:(*const u8,usize),b:*const str}const P:*const str=unsafe{U{a:(&B as *const u8,2)}.b};const N:u8=unsafe{*(P as *const u8)};fn main(){if N!=65{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
