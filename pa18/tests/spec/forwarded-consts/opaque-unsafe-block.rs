fn chunk<const N:usize>()->usize{N}macro_rules! pick{($e:expr)=>{chunk::<$e>()}}fn main(){let _=pick!(unsafe{7});}
