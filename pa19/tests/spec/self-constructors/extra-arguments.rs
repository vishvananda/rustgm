struct S<T>(T);impl<T>S<T>{fn bad(x:T)->Self{Self::<T>(x)}}fn main(){}
