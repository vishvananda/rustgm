fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static &'static u8{&&7}fn main(){check(**f()==7);}
