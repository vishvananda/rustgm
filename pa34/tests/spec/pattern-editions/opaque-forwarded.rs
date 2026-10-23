fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
macro_rules! id{($p:pat)=>{$p}}
macro_rules! outer{($p:pat)=>{id!($p)}}
fn main(){let outer!([mut x])=&[7u8];check(x==7);}
