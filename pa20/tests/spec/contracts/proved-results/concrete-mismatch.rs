trait Super{}
trait Sub:Super{}
trait Source{type Out;}
struct Wrap<I>(I);
impl<I> Super for Wrap<I> where I:Source<Out=u8>{}
impl<I:Source> Sub for Wrap<I>{}
fn main(){}
