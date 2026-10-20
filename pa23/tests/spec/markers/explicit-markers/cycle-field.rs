struct W;struct V(W);impl Unpin for W where V:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}
fn main(){}
