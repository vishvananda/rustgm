#![allow(dead_code)]
trait Value{type Out;}trait Make{fn make(&self)->impl Value<Out=u8>;}fn same<X>(_:X,_:X){}fn consumer<T:Make,U:Make>(x:&T,y:&U){same(x.make(),y.make());}fn main(){}
