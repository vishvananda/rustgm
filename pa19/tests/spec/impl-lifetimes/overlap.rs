trait K<R>{fn f(&self,r:R);}struct S;impl K<&S>for&S{fn f(&self,_:&S){}}impl K<&S>for&S{fn f(&self,_:&S){}}fn main(){}
