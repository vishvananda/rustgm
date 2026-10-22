trait K<R>{fn value(r:R)->R;}struct S;impl K<&S>for&S{fn value(r:&S)->&S{r}}fn escape<'a>(x:&'a S)->&'static S{<&S as K<&S>>::value(x)}fn main(){}
