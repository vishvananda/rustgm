trait Marker{fn value(&self)->u8;}
impl Marker for u8{fn value(&self)->u8{*self}}
impl<'a> Marker for &'a u8{fn value(&self)->u8{**self}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn op<'a>(_:&'a u8)->impl Marker+use<>{7u8}
fn main(){let result;{let x=7u8;result=op(&x);}check(result.value()==7);}
