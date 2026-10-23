struct S<T>(T);impl<T>S<T>{fn bad(x:T)->Self{fn inner()->S<u8>{Self(7)}inner()}}fn main(){}
