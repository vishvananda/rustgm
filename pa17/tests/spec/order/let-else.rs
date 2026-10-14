fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}enum E{A(D),B(D)}fn f(){let E::A(ref a)=E::B(D(1))else{trace(9);return;};}fn main(){f();}
