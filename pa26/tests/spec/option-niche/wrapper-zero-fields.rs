static S:u32=37;#[repr(transparent)]struct R([u8;0],&'static u32,());fn echo(v:Option<R>)->Option<R>{v}fn main(){check(match echo(Some(R([],&S,()))){Some(R(_,r,_))=>*r==37,None=>false});check(match echo(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
