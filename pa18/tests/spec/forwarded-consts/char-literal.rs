fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn mark<const C:char>()->char{C}macro_rules! pick{($c:literal)=>{mark::<$c>()}}fn main(){check(pick!('λ')=='λ');}
