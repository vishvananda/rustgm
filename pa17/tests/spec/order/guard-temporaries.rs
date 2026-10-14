fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){match 1{n if let g=(&D(1),D(2))&&false=>trace(8),_=>trace(9)}}
