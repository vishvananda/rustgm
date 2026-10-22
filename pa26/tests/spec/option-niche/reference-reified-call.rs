static S:u32=37;fn echo(v:Option<&u32>)->Option<&u32>{v}fn main(){let f:fn(Option<&u32>)->Option<&u32>=echo;check(match f(Some(&S)){Some(v)=>*v==37,None=>false});check(match f(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
