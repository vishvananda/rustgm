struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){let _=*self.0;}}fn see(_: &D)->bool{true}fn value()->bool{let x=1;see(&D(&x))}fn main(){let _=value();}
