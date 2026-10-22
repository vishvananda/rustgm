fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S;impl K<for<'a,'b>fn(&'a S,&'b S)->&'a S>for&S{fn accepts(&self,_:for<'a,'b>fn(&'a S,&'b S)->&'a S)->bool{true}}fn id<'a,'b>(_:&'a S,x:&'b S)->&'b S{x}fn main(){let a=S;check(K::accepts(&&a,id));}
