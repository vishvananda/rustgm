fn chunk<const N:i8>()->i8{N}macro_rules! pick{($e:expr)=>{chunk::<$e>()}}fn main(){let _=pick!(-7);}
