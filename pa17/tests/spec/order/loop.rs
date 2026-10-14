fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}fn f(mut n:u8){while n>0{let a=D(n);D(n+10);n-=1;if n>0{continue;}break;}}fn main(){f(2);trace(9);}
