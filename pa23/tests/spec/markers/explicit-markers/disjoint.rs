struct W<T>(T);impl Unpin for W<u8>{}impl Unpin for W<u16>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();need::<W<u16>>();}
fn main(){}
