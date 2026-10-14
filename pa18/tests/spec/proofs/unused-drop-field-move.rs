struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn bad<T>(x:D<T>)->T{x.0}fn main(){}
