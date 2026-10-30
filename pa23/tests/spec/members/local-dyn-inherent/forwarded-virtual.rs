fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}struct Number(u8);impl Value for Number{fn value(&self)->u8{self.0}}
impl dyn Value{fn forward(&self)->u8{self.value()}}
#[unsafe(no_mangle)]pub extern "C" fn observed_forward()->u8{let n=Number(7);let v:&dyn Value=&n;v.forward()}
#[unsafe(no_mangle)]pub extern "C" fn observed_item()->u8{let n=Number(9);let v:&dyn Value=&n;let f:fn(&(dyn Value+'static))->u8=<dyn Value>::forward;f(v)}
fn main(){check(observed_forward()==7);check(observed_item()==9);}
