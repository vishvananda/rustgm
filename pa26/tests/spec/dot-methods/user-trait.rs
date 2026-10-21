fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
trait AddAssign{fn add_assign(&mut self,r:u8);}struct W(u8);impl AddAssign for W{fn add_assign(&mut self,r:u8){self.0+=r+1;}}fn main(){let mut w=W(7);w.add_assign(1);verify(w.0==9);}
