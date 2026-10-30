trait Value{fn value(&self)->u8;}struct Number(u8);impl Value for Number{fn value(&self)->u8{self.0}}
trait Ready{}impl dyn Value{fn forward<T:Ready>(&self)->u8{self.value()}}fn main(){let n=Number(7);let x:&dyn Value=&n;let _=x.forward::<u8>();}
