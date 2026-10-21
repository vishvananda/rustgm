fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub fn eq(_:&u8,_:&u8)->bool{false}pub fn ne(_:&u8,_:&u8)->bool{false}}fn main(){check(!m::eq(&3,&3));check(!m::ne(&3,&4));}
