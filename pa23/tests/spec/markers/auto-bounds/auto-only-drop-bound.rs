fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn erase(x:&(dyn Send+Sync))->&dyn Send{x}fn main(){let x=7u8;let p=erase(&x);check(p as *const dyn Send as *const u8==&x as *const u8);}
