fn inc(v:Option<&mut u32>)->Option<&mut u32>{match v{Some(v)=>{*v+=1;Some(v)},None=>None}}fn main(){let mut v=36u32;check(match inc(Some(&mut v)){Some(v)=>*v==37,None=>false});check(match inc(None){None=>true,Some(_)=>false})}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
