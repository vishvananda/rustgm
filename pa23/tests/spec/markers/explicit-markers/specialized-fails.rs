struct W<T>(T);impl Unpin for W<u8>{}fn need<T:Unpin>(){}fn f(){need::<W<u16>>();}
fn main(){}
