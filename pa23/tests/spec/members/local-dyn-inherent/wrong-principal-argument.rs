trait Value<T>{fn value(&self)->T;}impl dyn Value<u8>{fn forward(&self)->u8{self.value()}}fn rejected(x:&(dyn Value<u16>+'static))->u8{x.forward()}fn main(){}
