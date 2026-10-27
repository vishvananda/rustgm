fn chunk<const N:usize>()->usize{N}
macro_rules! pick{($n:literal)=>{chunk::<$n u8>()}}fn main(){let _=pick!(4);}
