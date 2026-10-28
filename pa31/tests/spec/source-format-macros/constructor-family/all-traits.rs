use std::fmt;
struct Probe;
macro_rules! implement {($($name:ident),*)=>{$(impl fmt::$name for Probe{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str(stringify!($name))}})*}}
implement!(Display,Debug,Octal,LowerHex,UpperHex,Pointer,Binary,LowerExp,UpperExp);
fn main(){println!("{0}|{0:?}|{0:o}|{0:x}|{0:X}|{0:p}|{0:b}|{0:e}|{0:E}",Probe);}
