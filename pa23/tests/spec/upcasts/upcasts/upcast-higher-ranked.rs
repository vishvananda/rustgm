fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{fn a(&self,x:&'a u8)->&'a u8;}trait B:for<'a>A<'a>{}impl<'a>A<'a>for u8{fn a(&self,x:&'a u8)->&'a u8{x}}impl B for u8{}fn base(x:&dyn B)->&dyn for<'a>A<'a>{x}fn main(){let v=7u8;let x=9u8;check(*base(&v).a(&x)==9);}
