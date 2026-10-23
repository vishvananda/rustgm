trait Marker{fn value(&self)->u8;}
impl Marker for u8{fn value(&self)->u8{*self}}
impl<'a> Marker for &'a u8{fn value(&self)->u8{**self}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Holder;impl Holder{fn op<'a>(&self,x:&'a u8)->impl Marker{x}}
fn main(){let x=7u8;let h=Holder;check(h.op(&x).value()==7);}
