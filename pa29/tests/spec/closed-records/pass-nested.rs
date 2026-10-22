fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=one::nested(4);check(p.tag==3);check(one::take(p.pair)==14);}
