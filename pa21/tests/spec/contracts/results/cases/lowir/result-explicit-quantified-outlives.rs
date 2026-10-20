trait Family where for<'a>Self::Item:'a{type Item;}impl Family for u16{type Item=u8;}fn need<'a,X:'a>(){}fn f<T:Family>(){need::<'static,<T as Family>::Item>();}fn main(){f::<u16>();}
