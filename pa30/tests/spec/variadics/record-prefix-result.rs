#[repr(C)]struct Pair{a:u64,b:f64}unsafe extern "C"{fn record_result(x:Pair,...)->Pair;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p=unsafe{record_result(Pair{a:7,b:2.5},3u64,1.5f64)};check(p.a==10&&p.b==4.0);}
