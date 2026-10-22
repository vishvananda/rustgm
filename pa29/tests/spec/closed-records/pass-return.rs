fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=one::pair(4);check(p.x==5&&p.y==9);check(one::take(p)==14);}
