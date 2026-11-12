fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;fn get(&self)->&Self::Item;}
trait E<R:?Sized>{fn e(&self,o:&R)->bool;}
struct A(u8);struct B(u16);
impl It for A{type Item=u8;fn get(&self)->&u8{&self.0}}
impl It for B{type Item=u16;fn get(&self)->&u16{&self.0}}
impl E<u16> for u8{fn e(&self,o:&u16)->bool{*self as u16==*o}}
fn f<X:It,Y:It>(a:&X,b:&Y)->bool where X::Item:E<Y::Item>{X::Item::e(a.get(),b.get())}
fn main(){check(f(&A(3),&B(3)));check(!f(&A(3),&B(4)));}
