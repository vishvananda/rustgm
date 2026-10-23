trait Marker{fn value(&self)->u8;}
impl Marker for u8{fn value(&self)->u8{*self}}
impl<'a> Marker for &'a u8{fn value(&self)->u8{**self}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Holder<'a>(&'a u8);impl<'a> Holder<'a>{fn op(&self)->impl Marker{7u8}}
fn main(){let result;{let x=7u8;let h=Holder(&x);result=h.op();}check(result.value()==7);}
