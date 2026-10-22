fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=va_provider::Pair{a:7,b:2.5};let q=unsafe{va_provider::record_result(p,11u64,1.5f64)};check(q.a==18&&q.b==4.0);}
