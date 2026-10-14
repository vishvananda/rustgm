fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T>=for<'a> fn(&'a u8,T)->T;fn keep<'a,'b>(_:&'a u8,x:&'b u8)->&'b u8{x}fn apply<'b>(f:A<&'b u8>,x:&'b u8)->&'b u8{let y=7;f(&y,x)}fn main(){let z=31;check(*apply(keep,&z)==31);}
