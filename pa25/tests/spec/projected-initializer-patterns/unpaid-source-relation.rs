trait Pick<'a>{type Out;const V:Self::Out;}struct S<'a,'b,T>(&'a u8,&'b u8,T);impl<'a,'b,T:Pick<'a,Out=&'a u8>>S<'a,'b,T>{const C:&'b u8={let n=T::V;n};}fn main(){}
