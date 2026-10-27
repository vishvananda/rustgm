fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn truth<const B:bool>()->bool{B}macro_rules! pick{($b:literal)=>{truth::<$b>()}}fn main(){check(pick!(true));check(!pick!(false));}
