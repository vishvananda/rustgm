fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Element{type Out;}impl Element for u8{type Out=u8;}fn f<'b,'c,T:Element>(x:&'c &'b T::Out)->&'b T::Out{(|&s|s)(x)}
fn main(){let x=7u8;let p=&x;check(*f::<u8>(&p)==7);}
