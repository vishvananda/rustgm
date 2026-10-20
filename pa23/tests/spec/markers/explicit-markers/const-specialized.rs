struct W<const N:usize>;impl Unpin for W<3>{}fn need<T:Unpin>(){}fn f(){need::<W<3>>();}
fn main(){}
