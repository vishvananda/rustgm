#![allow(dead_code)]
trait Value{type Out;}trait Make{fn make(&self)->impl Value<Out=u8>;}fn consumer<T:Make>(x:&T)->impl Value<Out=u8>+'static{x.make()}fn main(){}
