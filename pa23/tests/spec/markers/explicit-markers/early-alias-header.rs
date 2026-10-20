struct W(*const u8);trait A{type Out;}impl<T:Send> A for T{type Out=u8;}type Output=<W as A>::Out;type Alias=W;unsafe impl Send for Alias{}fn f(_:Output){}
fn main(){}
