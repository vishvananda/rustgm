trait Marker{}
trait Super{}
trait Sub:Super{}
trait Source{type Out;}
struct Wrap<I>(I);
impl<I,U> Super for Wrap<I> where I:Source<Out=U>,U:Marker{}
impl<I:Source> Sub for Wrap<I>{}
fn main(){}
