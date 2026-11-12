enum Option<T>{None,Some(T)}
enum ControlFlow<B,C=()>{Continue(C),Break(B)}
impl<B,C> ControlFlow<B,C>{fn break_value(self)->Option<B>{match self{ControlFlow::Break(x)=>Option::Some(x),ControlFlow::Continue(_)=>Option::None}}}
trait Try{type Output;}
impl<B,C> Try for ControlFlow<B,C>{type Output=C;}
trait Iterator{fn try_for_each<F,R>(&mut self,f:F)->R where F:FnMut(u8)->R,R:Try<Output=()>{loop{}}}
fn code<I:Iterator>(i:&mut I)->Option<u8>{i.try_for_each(ControlFlow::Break).break_value()}
