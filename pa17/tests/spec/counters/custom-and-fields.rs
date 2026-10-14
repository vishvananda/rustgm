fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}
struct Outer<'a>{own:&'a mut u8,field:D<'a>}impl Drop for Outer<'_>{fn drop(&mut self){*self.own+=10;}}fn main(){let mut a=0;let mut b=0;{let _o=Outer{own:&mut a,field:D(&mut b)};}check(a==10&&b==1);}
