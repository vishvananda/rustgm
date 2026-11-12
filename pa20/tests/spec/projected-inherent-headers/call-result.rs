fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Try{type Output;fn from_output(o:Self::Output)->Self;}
enum Flow<B,C>{Continue(C),Break(B)}
impl<R:Try> Flow<R,R::Output>{fn into_try(self)->R{match self{Flow::Continue(v)=>R::from_output(v),Flow::Break(v)=>v}}fn proceeds(&self)->bool{match self{Flow::Continue(_)=>true,Flow::Break(_)=>false}}}
struct Num(u8);impl Try for Num{type Output=u8;fn from_output(o:u8)->Self{Num(o+1)}}
fn step<A,R:Try<Output=A>,F:FnOnce(A)->Flow<R,A>>(a:A,f:F)->R{f(a).into_try()}
fn main(){check(step(10,|x:u8|Flow::<Num,u8>::Continue(x)).0==11);check(step(10,|x:u8|Flow::<Num,u8>::Break(Num(x+2))).0==12);}
