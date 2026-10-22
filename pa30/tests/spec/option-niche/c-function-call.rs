fn f()->u32{37}extern "C" fn echo(v:Option<fn()->u32>)->Option<fn()->u32>{v}fn main(){check(match echo(Some(f as fn()->u32)){Some(f)=>f()==37,None=>false});check(match echo(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
