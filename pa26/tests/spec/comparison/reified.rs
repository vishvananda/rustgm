fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let e:for<'a,'b> fn(&'a u8,&'b u8)->bool=<u8 as PartialEq>::eq;let n:for<'a,'b> fn(&'a u8,&'b u8)->bool=<u8 as PartialEq>::ne;check(e(&3,&3));check(n(&3,&4));}
