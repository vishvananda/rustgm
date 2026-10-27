fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn chunk<const N:usize>()->usize{N}
macro_rules! inner{($n:literal)=>{chunk::<$n>()}}macro_rules! outer{($n:literal)=>{inner!($n)}}fn main(){check(outer!(7)==7usize);}
