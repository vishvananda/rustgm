type F=extern "C" fn(u32)->u32;extern "C" fn inc(x:u32)->u32{x+1}unsafe extern "C"{fn fn_echo(v:Option<F>)->Option<F>;}fn main(){check(match unsafe{fn_echo(Some(inc))}{Some(f)=>f(36)==37,None=>false});check(match unsafe{fn_echo(None)}{None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
