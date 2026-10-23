const B:&[u8]=&[255u8];union U{a:*const str,b:*const str}const P:*const str=unsafe{U{a:B as *const [u8] as *const str}.b};const N:u8=unsafe{*(P as *const u8)};fn main(){if N!=255{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
