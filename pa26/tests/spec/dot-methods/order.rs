fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
use core::ops::AddAssign;fn mark<'a>(x:&'a mut u8,y:&u8)->&'a mut u8{*x=*y+1;x}fn rhs(y:&mut u8)->u8{*y+=10;1}fn main(){let mut x=0u8;let mut y=7u8;mark(&mut x,&y).add_assign(rhs(&mut y));verify(x==9);verify(y==17);}
