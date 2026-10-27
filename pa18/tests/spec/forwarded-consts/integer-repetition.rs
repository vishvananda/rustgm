fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn chunk<const N:usize>()->usize{N}
macro_rules! sum{($($n:literal)+)=>{0usize$(+chunk::<$n>())+}}fn main(){check(sum!(4 2 1)==7usize);}
