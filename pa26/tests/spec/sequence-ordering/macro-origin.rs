fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
macro_rules! order{($a:expr,$b:expr)=>{$a<$b}}fn main(){check(order!([1u8],[2u8]));}