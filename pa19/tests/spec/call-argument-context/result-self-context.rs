fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn make(r:R)->Self;}struct S(u8);impl K<fn(&S)->&S>for S{fn make(r:fn(&S)->&S)->Self{S(r(&S(3)).0)}}fn id(x:&S)->&S{x}fn main(){let a:S=K::make(id);check(a.0==3);}
