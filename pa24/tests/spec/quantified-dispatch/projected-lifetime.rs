fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick<'a>{type Out:Copy+'a;}impl<'a>Pick<'a> for u8{type Out=&'a u8;}fn run<'a,T:Pick<'a>>(x:T::Out)->u8{let f=move|a:&u8|{let y=x;let _=y;*a};let p:&dyn Fn(&u8)->u8=&f;p(&7)}fn main(){let x=3;check(run::<u8>(&x)==7);}
