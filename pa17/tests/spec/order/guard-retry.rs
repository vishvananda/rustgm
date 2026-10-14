fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn main(){let mut attempt=0;match 7{n@(0..=9|5..=15) if let g=D(1)&&{attempt+=1;attempt==2}=>trace(8),_=>trace(9)}}
