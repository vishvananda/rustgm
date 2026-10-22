fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
macro_rules! same{($a:expr,$b:expr)=>{$a==$b}}fn main(){check(same!([1u8],[1u8]));check(same!([2u8],[2u8]));}
