fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type U=u32;fn main(){let mut x:U=3;let y:U=4;x+=&y;check(x==7);}
