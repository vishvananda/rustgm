#![allow(dead_code)]
trait Value{type Out;}trait Make{fn make(&self)->impl Value<Out=u8>;fn other(&self)->impl Value<Out=u8>;}fn same<X>(_:X,_:X){}fn consumer<T:Make>(x:&T){same(x.make(),x.other());}fn main(){}
