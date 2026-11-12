unsafe extern "C" { fn mark(step:u8); }
enum Option<T>{None,Some(T)}
enum ControlFlow<B,C=()>{Continue(C),Break(B)}
impl<B,C> ControlFlow<B,C>{fn break_value(self)->Option<B>{unsafe{mark(2)};match self{ControlFlow::Break(x)=>Option::Some(x),ControlFlow::Continue(_)=>Option::None}}}
trait Try{type Output;}
impl<B,C> Try for ControlFlow<B,C>{type Output=C;}
trait Iterator{type Item;fn try_for_each<F,R>(&mut self,f:F)->R where F:FnMut(Self::Item)->R,R:Try<Output=()>{loop{}}}
struct Once{value:u8}
impl Iterator for Once{type Item=u8;fn try_for_each<F,R>(&mut self,mut f:F)->R where F:FnMut(Self::Item)->R,R:Try<Output=()>{unsafe{mark(1)};f(self.value)}}
fn collect<I:Iterator>(i:&mut I)->Option<I::Item>{i.try_for_each(ControlFlow::Break).break_value()}
#[unsafe(no_mangle)]pub extern "C" fn code()->u8{let mut iter=Once{value:23};match collect(&mut iter){Option::Some(value)=>value,Option::None=>0}}
