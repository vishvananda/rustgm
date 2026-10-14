fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}enum E{A(D,D),B(D,D)}fn f(e:E){match e{E::A(x,y)|E::B(y,x)=>trace(9)}}fn main(){f(E::A(D(1),D(2)));f(E::B(D(1),D(2)));}
