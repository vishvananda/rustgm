macro_rules! bad{()=>{fn unused<T>(x:T){let a=x;let b=x;}}}bad!();fn main(){}
