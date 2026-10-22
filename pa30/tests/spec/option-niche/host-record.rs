static S:u32=37;#[repr(C)]struct R{a:Option<&'static u32>,b:u64}unsafe extern "C"{fn record_echo(v:R)->R;}fn main(){let a=unsafe{record_echo(R{a:Some(&S),b:18})};check(a.b==19);check(match a.a{Some(v)=>*v==37,None=>false});let b=unsafe{record_echo(R{a:None,b:22})};check(b.b==23);check(match b.a{None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
