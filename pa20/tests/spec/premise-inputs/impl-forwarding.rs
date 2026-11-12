fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;fn get(&self)->&Self::Item;}
trait E<R:?Sized>{fn e(&self,o:&R)->bool;}
struct A(u8);struct B(u16);
impl It for A{type Item=u8;fn get(&self)->&u8{&self.0}}
impl It for B{type Item=u16;fn get(&self)->&u16{&self.0}}
impl E<u16> for u8{fn e(&self,o:&u16)->bool{*self as u16==*o}}
struct W<P>(P);
impl<X:It,Y:It> E<W<Y>> for W<X> where X::Item:E<Y::Item>{fn e(&self,o:&W<Y>)->bool{X::Item::e(self.0.get(),&o.0.get())}}
fn main(){check(W(A(3)).e(&W(B(3))));check(!W(A(3)).e(&W(B(4))));}
