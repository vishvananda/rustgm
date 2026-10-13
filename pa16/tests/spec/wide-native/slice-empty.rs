fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(s:&[u8])->&[u8]{s}fn main(){let _=id(&[]);let _=id(b"");}
