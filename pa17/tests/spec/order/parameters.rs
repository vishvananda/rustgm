fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn f((x,_):(D,D),(_,y):(D,D)){trace(9);}fn main(){f((D(0),D(1)),(D(2),D(3)));}
