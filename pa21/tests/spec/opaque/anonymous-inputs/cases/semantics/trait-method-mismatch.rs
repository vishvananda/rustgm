trait A{fn f(x:impl Copy);}struct S;impl A for S{fn f<T:Copy>(x:T){}}fn main(){}
