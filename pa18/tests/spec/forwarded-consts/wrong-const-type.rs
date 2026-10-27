fn chunk<const N:usize>()->usize{N}
macro_rules! pick{($n:literal)=>{chunk::<$n>()}}fn main(){let _=pick!(true);}
