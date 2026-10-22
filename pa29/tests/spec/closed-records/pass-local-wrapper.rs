struct Local{pair:one::Pair}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=Local{pair:one::pair(4)};check(one::take(p.pair)==14);}
