fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
use core::ops::AddAssign;struct W(u8);impl core::ops::AddAssign<u8> for W{fn add_assign(&mut self,r:u8){self.0+=r;}}
fn main(){let mut w=W(7);{let mut f=||w.add_assign(1);f();f();}verify(w.0==9);}
