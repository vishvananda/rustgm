fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a:[u8;0]=[];let s=&a as &[u8];check(match s{[]=>true,_=>false});}
