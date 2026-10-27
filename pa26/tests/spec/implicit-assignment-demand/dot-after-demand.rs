fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=3u32;let y=4u32;x+=&y;x.add_assign(&y);check(x==11);}
