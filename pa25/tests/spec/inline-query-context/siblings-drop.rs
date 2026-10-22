fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a> for Tag{type Out=&'a u8;const V:Self::Out=&7;}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{const C:T::Out={let _=const{let n=T::V;n};const{let n=T::V;n}};}fn get<'a>(_:&'a u8)->&'a u8{S::<'a,Tag>::C}fn main(){let n=7;check(*get(&n)==7);}
