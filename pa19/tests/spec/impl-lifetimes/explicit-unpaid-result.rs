trait K<R>{fn value(self,r:R)->R;}struct S;impl<'a,'b>K<&'b S>for&'a S{fn value(self,_:&'b S)->&'b S{self}}fn main(){}
