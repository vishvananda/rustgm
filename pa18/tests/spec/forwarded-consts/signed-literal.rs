fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn signed<const N:i8>()->i8{N}macro_rules! pick{($n:literal)=>{signed::<$n>()}}fn main(){check(pick!(-7)==-7i8);}
