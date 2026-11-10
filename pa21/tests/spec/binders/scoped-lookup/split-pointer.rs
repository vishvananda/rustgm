struct Split<F>(F);
impl<F:for<'a>FnMut(&'a u8)->bool>Split<F>{fn new(f:F)->Self{Self(f)}}
trait Mention{fn new()->u16;}
fn predicate(_: &u8)->bool{true}
fn split<F:FnMut(&u8)->bool>(f:F)->Split<F>{Split::new(f)}
fn main(){let _=split(predicate as fn(&u8)->bool);}
