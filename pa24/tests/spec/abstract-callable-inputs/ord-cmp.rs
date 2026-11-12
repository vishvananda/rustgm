fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use std::cmp::Ordering;
trait It{type Item;}
impl It for u8{type Item=u16;}
fn by<T,F:FnMut(&T,&T)->Ordering>(x:T,y:T,mut f:F)->T{match f(&x,&y){Ordering::Greater=>x,_=>y}}
fn pick<I:It>(x:I::Item,y:I::Item)->I::Item where I::Item:Ord{by(x,y,Ord::cmp)}
fn main(){check(pick::<u8>(7,9)==9);check(pick::<u8>(9,7)==9);}
