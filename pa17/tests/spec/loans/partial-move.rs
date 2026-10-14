struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn observe(_:u8){}fn f(){let mut a=0;let mut b=0;let p=(D(&mut a),D(&mut b));{let _a=p.0;}observe(a);}fn main(){}
