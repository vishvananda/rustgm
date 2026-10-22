static S:u32=37;extern "C" fn echo(v:Option<&u32>)->Option<&u32>{v}fn main(){let f:extern "C" fn(Option<&u32>)->Option<&u32>=echo;check(match f(Some(&S)){Some(r)=>*r==37,None=>false});check(match f(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
