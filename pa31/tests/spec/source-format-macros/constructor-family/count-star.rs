use std::fmt;
struct Probe;
impl fmt::Display for Probe{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{let w=f.width();let p=f.precision();write!(f,"w{w:?}:p{p:?}")}}
fn main(){println!("{:.*}|{}",2usize,Probe,Probe);}
