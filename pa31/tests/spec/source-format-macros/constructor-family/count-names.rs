use std::fmt;
struct Probe;
impl fmt::Display for Probe{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{let w=f.width();let p=f.precision();write!(f,"w{w:?}:p{p:?}")}}
fn main(){let width=5usize;let precision=2usize;let value=Probe;println!("{value:width$.precision$}");}
