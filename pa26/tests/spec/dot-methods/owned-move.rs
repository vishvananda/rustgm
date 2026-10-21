fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
use core::ops::Add;struct W(u8);impl core::ops::Add for W{type Output=u8;fn add(self,r:W)->u8{self.0+r.0}}fn main(){let w=W(7);let _=w.add(W(2));verify(w.0==7);}
