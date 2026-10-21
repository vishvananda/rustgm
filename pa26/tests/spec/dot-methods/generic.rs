fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn update<T:core::ops::AddAssign<u8>>(x:&mut T){x.add_assign(2);}struct W(u8);impl core::ops::AddAssign<u8> for W{fn add_assign(&mut self,r:u8){self.0+=r;}}
fn main(){let mut w=W(7);update(&mut w);verify(w.0==9);}
