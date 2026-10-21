fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn bad()->&'static u8{let n=7;let r=&n;let f=move||r;f()}fn main(){}
