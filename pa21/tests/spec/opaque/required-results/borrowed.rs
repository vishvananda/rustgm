#![allow(dead_code)]
trait Value{type Out;}trait Make{fn make(&self)->impl Value<Out=u8>;}fn require<F:Value<Out=u8>>(_:F){}fn consumer<T:Make>(x:&T){require(x.make());}fn main(){}
