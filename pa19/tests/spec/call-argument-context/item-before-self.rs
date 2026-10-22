fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(r:R,s:&Self)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:&&S)->bool{r(s).0==s.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(id,&&a));}
