fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
macro_rules! bind{($name:ident)=>{[mut $name]}}
fn main(){let bind!(x)=&[7u8];check(x==7);}
