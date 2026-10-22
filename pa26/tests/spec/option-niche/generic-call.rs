static S:u32=37;static T:u8=19;fn echo<T>(v:Option<&T>)->Option<&T>{v}fn main(){check(match echo(Some(&S)){Some(v)=>*v==37,None=>false});check(match echo(Some(&T)){Some(v)=>*v==19,None=>false});check(match echo::<u32>(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
