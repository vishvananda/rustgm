union U{a:&'static str,b:&'static str}const S:&str=unsafe{U{a:""}.b};const B:&[u8]=unsafe{*(&raw const S as *const &[u8])};const N:usize=B.len();fn main(){if N!=0{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
