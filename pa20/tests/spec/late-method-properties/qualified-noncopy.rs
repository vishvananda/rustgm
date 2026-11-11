trait Family{type Out;}struct N;struct Non;impl Family for Non{type Out=N;}struct Byte;impl Family for Byte{type Out=u8;}
trait A<U>{}
struct S<T,U,V>(T,U,V);
impl<T:A<U>,U:Copy>S<T,U,u8>{fn run(self){}}
impl<T:A<U>,U>S<T,U,u16>{fn run(self){}}
fn pick<T:A<<Non as Family>::Out>>(){S::<T,_,_>::run(loop{})}
fn main(){}
