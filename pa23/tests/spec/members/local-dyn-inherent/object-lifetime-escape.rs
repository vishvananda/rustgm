trait Value{fn value(&self)->u8;}struct Number(u8);impl Value for Number{fn value(&self)->u8{self.0}}
impl dyn Value+'static{fn forward(&self)->u8{self.value()}}fn rejected<'a>(x:&'a(dyn Value+'a))->u8{x.forward()}fn main(){}
