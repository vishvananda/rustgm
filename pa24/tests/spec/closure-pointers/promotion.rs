fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let fs:&[fn(u8)->u8;2]=&[|x|x+1,|x|x+2];check(fs[0](7)==8);check(fs[1](7)==9);}