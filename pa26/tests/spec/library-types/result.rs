fn get(x:Result<u8,u16>)->u16{match x{Ok(x)=>x as u16,Err(x)=>x}}fn main(){let _=get(Ok(7));let _=get(Err(9));}
