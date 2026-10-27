fn accept<T>(){}macro_rules! pick{($e:expr)=>{accept::<$e>()}}fn main(){pick!(7);}
