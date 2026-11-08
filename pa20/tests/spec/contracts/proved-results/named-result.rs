trait Super{}
trait Sub:Super{}
trait Source{type Out;}
struct Wrap<I>(I);
impl<I,U> Super for Wrap<I> where I:Source<Out=U>{}
impl<I,U> Sub for Wrap<I> where I:Source<Out=U>{}
fn main(){}
