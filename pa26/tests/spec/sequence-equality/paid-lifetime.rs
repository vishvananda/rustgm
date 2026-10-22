fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct R<'a>{f:fn(&'a u8)->&'a u8}impl<'a:'b,'b>PartialEq<R<'b>> for R<'a>{fn eq(&self,_:&R<'b>)->bool{true}}fn id(x:&u8)->&u8{x}fn eq<'a:'b,'b>(a:&[R<'a>],b:&[R<'b>])->bool{a==b}fn main(){let a=[R{f:id}];let b=[R{f:id}];check(eq(&a,&b));}
