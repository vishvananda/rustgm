fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get<'b>{type Out;fn get(&self,x:&'b u8)->Self::Out;}struct Holder<'a>(&'a u8);impl<'a,'b> Get<'b> for Holder<'a>{type Out=&'a u8;fn get(&self,_:&'b u8)->Self::Out{self.0}}fn run<'a>(x:&'a u8)->u8{let f=Holder(x);let p:&dyn for<'b>Get<'b,Out=&'a u8>=&f;*p.get(&0)}fn main(){let x=7;check(run(&x)==7);}
