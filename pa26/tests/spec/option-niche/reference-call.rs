static S:u32=37;fn echo(v:Option<&u32>)->Option<&u32>{v}fn main(){check(match echo(Some(&S)){Some(v)=>*v==37,None=>false});check(match echo(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
