struct W;struct V;type Alias=V;impl Unpin for Alias{}fn need<T:Unpin>(){}fn f(){need::<W>();}
fn main(){}
