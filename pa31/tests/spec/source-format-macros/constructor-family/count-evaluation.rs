use std::fmt;
struct Probe;
impl fmt::Display for Probe{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{let w=f.width();let p=f.precision();write!(f,"w{w:?}:p{p:?}")}}
fn count(n:usize)->usize{print!("c{n};");n}
fn value()->Probe{print!("v;");Probe}
fn main(){let text=format!("{value:width$.precision$}",width=count(2),value=value(),precision=count(3));println!("{text}");}
