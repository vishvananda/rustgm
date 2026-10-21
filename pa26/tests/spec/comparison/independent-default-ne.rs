fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a,'b> PartialEq<S<'b>> for S<'a>{fn eq(&self,r:&S<'b>)->bool{*self.0==*r.0}}fn different<'a,'b>(a:S<'a>,b:S<'b>)->bool{let v=a!=b;let _=(a.0,b.0);v}fn main(){let a=4;let x=S(&a);let v;{let b=5;let y=S(&b);v=different(x,y);}check(v);}
