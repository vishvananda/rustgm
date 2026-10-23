fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let [&(left,right)]=&[&(3u8,4u8)];check(left+right==7);}
