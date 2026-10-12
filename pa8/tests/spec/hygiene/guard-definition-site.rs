macro_rules! choose{($x:expr,$body:expr)=>{match $x{x if let hidden=x=>$body,_=>0}}}fn main(){let _=choose!(7u8,hidden);}
