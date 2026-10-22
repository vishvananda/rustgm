fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq(a:&[u8],b:&[u8])->bool{a==b}fn main(){check(eq(&[1,2],&[1,2]));check(!eq(&[1,2],&[1]));check(eq(&[],&[]));}
