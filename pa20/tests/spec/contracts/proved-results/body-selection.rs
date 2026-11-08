trait Super{}
trait Source{type Out;}
struct Wrap<I>(I);
impl<I,U> Super for Wrap<I> where I:Source<Out=U>{}
fn need<T:Super>(){}
fn call<I:Source>(){need::<Wrap<I>>();}
struct Short;
impl Source for Short{type Out=u8;}
fn main(){call::<Short>();}
