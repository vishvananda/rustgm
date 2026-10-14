fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}
fn main(){let mut a=0;let mut b=0;{let [first,..]=[D(&mut a),D(&mut b)];let _moved=first;}check(a==1&&b==1);}
