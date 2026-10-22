trait K<R>{fn value(self,r:R)->R;}struct S;impl K<&S>for&S{fn value(self,_:&S)->&S{self}}fn main(){}
