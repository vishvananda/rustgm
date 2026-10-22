fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pick<const N:usize>(_:&[u8;1])->usize where [u8;1]:PartialEq<[u8;N]>{N}fn main(){check(pick(&[1])==1);}
