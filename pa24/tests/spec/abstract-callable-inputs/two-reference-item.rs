fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;}
impl It for u8{type Item=u16;}
trait Num{fn num(&self)->u16;}
impl Num for u16{fn num(&self)->u16{*self}}
fn by<T,F:FnMut(&T,&T)->u16>(x:T,y:T,mut f:F)->u16{f(&x,&y)}
fn sum<S:Num>(a:&S,b:&S)->u16{a.num()+b.num()}
fn pick<I:It>(x:I::Item,y:I::Item)->u16 where I::Item:Num{by(x,y,sum)}
fn main(){check(pick::<u8>(7,9)==16);}
