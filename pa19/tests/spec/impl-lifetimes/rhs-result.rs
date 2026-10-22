fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn value(r:R)->R;}struct S(u8);impl K<&S>for&S{fn value(r:&S)->&S{r}}fn main(){let a=S(5);check(<&S as K<&S>>::value(&a).0==5);}
