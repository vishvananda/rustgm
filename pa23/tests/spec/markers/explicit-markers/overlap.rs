struct W<T>(T);impl<T> Unpin for W<T>{}impl Unpin for W<u8>{}
fn main(){}
