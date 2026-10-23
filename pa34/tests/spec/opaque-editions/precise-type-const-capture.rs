trait Marker{fn value(&self)->u8;}
impl Marker for u8{fn value(&self)->u8{*self}}
impl<'a> Marker for &'a u8{fn value(&self)->u8{**self}}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn op<T:Marker,const N:usize>(_:&u8,x:T)->impl Marker+use<T,N>{x}
fn main(){let result;{let x=0u8;result=op::<u8,3>(&x,7u8);}check(result.value()==7);}
