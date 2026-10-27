fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn rhs(p:&u32)->&u32{p}fn main(){let mut x=3u32;let y=4u32;x+=rhs(&y);check(x==7);}
