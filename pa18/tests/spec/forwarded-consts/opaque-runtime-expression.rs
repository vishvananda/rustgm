fn chunk<const N:usize>()->usize{N}
macro_rules! pick{($e:expr)=>{chunk::<$e>()}}fn main(){let value=4usize;let _=pick!(value);}
