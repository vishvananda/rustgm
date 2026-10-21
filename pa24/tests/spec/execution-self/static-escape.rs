fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:Fn()->&'static u8>(f:F)->F{f}fn main(){let n=7u8;let r=&n;let _f=infer(move||r);}
