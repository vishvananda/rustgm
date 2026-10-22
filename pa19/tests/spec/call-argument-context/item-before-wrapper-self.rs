fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);trait K<R>:Sized{fn accepts(r:R,s:Wrap<Self>)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(r:fn(&S)->&S,s:Wrap<Self>)->bool{r(s.0).0==s.0.0}}fn id(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(id,Wrap(&a)));}
