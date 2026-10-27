fn chunk<const N:usize>()->usize{N}
macro_rules! pick{($e:expr)=>{chunk::<$e>()}}fn main(){let _=pick!({1+2});}
