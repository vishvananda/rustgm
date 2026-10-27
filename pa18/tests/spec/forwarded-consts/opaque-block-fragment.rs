fn chunk<const N:usize>()->usize{N}macro_rules! pick{($b:block)=>{chunk::<$b>()}}fn main(){let _=pick!({7});}
