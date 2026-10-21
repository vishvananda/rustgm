fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a,'b> PartialEq<S<'b>> for S<'a>{fn eq(&self,r:&S<'b>)->bool{*self.0==*r.0}}fn equal<'a,'b>(a:S<'a>,b:S<'b>)->bool{a==b}fn main(){let a=4;let x=S(&a);let v;{let b=4;let y=S(&b);v=equal(x,y);}check(v);}
