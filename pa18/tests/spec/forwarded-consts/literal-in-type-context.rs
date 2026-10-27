macro_rules! bad{($n:literal)=>{fn value(_: $n){}}}bad!(4);fn main(){}
