use std::fmt;
struct Probe;
impl fmt::Display for Probe{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{let w=f.width();let p=f.precision();write!(f,"w{w:?}:p{p:?}")}}
type Count=usize;
fn main(){let width:Count=0;let precision:Count=0;println!("{1:0$.2$}",width,Probe,precision);}
