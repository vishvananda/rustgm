macro_rules! choose{($x:expr,$v:ident)=>{match $x{x if let $v=x+1=>$v,_=>0}}}fn main(){let n=100u8;let v=choose!(7u8,n);let _=n+v;}
