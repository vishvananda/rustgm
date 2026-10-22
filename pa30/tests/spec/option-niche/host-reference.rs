static S:u32=37;unsafe extern "C"{fn ptr_echo(v:Option<&u32>)->Option<&u32>;}fn main(){check(match unsafe{ptr_echo(Some(&S))}{Some(r)=>*r==37,None=>false});check(match unsafe{ptr_echo(None)}{None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
