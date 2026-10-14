fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn f(flag:bool){let p=(D(1),D(2),D(3));if flag{let a=p.0;}trace(9);}fn main(){f(false);f(true);}
