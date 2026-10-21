fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait PartialOrd{fn lt(&self,r:&Self)->bool;}struct S;impl PartialOrd for S{fn lt(&self,_:&Self)->bool{true}}fn main(){let _=S<S;}
