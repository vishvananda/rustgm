fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->&[u8]{let [rest @ ..]=s;rest}fn main(){check(f(&[7,9])[1]==9);let _=f(&[]);}
