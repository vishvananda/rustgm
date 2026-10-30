fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}struct Number(u8);impl Value for Number{fn value(&self)->u8{self.0}}
impl<'a>dyn Value+'a{fn forward(&self)->u8{self.value()}}fn apply<'a>(x:&'a(dyn Value+'a))->u8{x.forward()}fn main(){let n=Number(7);check(apply(&n)==7);}
