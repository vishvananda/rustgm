fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=va_bridge::Pair{a:7,b:2.5};check(unsafe{va_provider::record_tail(0,p)}==1);}
