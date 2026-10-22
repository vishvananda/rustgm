fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&u8)->bool{match x{7=>true,_=>false}}fn main(){check(f(&7));check(!f(&8));}
