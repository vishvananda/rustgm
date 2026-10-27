fn check(ok:bool){if !ok{let _=fail(0u8);}}fn fail(x:u8)->u8{1/x}
fn chunk<const N:usize>()->usize{N}
macro_rules! pick{($n:literal)=>{chunk::<{$n+1}>()}}fn main(){check(pick!(6)==7usize);}
