fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<'a>(x:&'a [u8])->&'a [u8]{x}fn main(){let a=[3,7,9];let s=id(&a);check(s[2]==9);}
