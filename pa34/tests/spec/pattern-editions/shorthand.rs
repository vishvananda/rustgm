fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{v:u8}
fn main(){let S{mut v}=&S{v:7};check(v==7);v=9;check(v==9);}
