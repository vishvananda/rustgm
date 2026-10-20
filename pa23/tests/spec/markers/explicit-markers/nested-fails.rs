struct W<T>(T);struct Outer(W<u16>);impl Unpin for W<u8>{}fn need<T:Unpin>(){}fn f(){need::<Outer>();}
fn main(){}
