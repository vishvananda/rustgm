struct W;struct V;impl Unpin for W where V:Unpin{}impl Unpin for V where W:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}
fn main(){}
