fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let [mut whole @ part]=&[7u8];check(whole==7);check(*part==7);whole=9;check(whole==9);}
