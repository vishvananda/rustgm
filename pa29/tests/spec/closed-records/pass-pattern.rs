fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let one::Pair{x,y}=one::pair(4);check(x==5&&y==9);}
