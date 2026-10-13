fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(i:usize)->&'static u8{&[1,2][i]}fn main(){let _=f(2);}
