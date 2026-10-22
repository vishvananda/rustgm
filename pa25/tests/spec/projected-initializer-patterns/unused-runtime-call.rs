fn runtime()->u8{7}trait Pick{const V:u8;}struct S<T>(T);impl<T:Pick>S<T>{const C:u8={let n=runtime();n};}fn main(){}
