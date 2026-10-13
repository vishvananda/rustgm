fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=7u8;let b=9u8;let rr=&mut &a;*rr=&b;check(**rr==9);}
