fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Num{fn num(&self)->u16;}
impl Num for u16{fn num(&self)->u16{*self}}
fn by<T,F:FnMut(&T)->u16>(x:T,mut f:F)->u16{f(&x)}
fn pick<T:Num>(x:T)->u16{by(x,Num::num)}
fn main(){check(pick(7u16)==7);}
