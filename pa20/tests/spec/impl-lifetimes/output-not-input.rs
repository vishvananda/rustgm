trait K<R>{type Out;}struct S;impl K<&S>for&S{type Out=&S;}fn main(){}
