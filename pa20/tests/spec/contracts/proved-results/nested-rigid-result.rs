trait Other{type Item;}
trait Super{}
trait Sub:Super{}
trait Source{type Out;}
struct Pair<A>(A);
struct Wrap<I>(I);
impl<I,U> Super for Wrap<I> where I:Source<Out=Pair<U>>{}
impl<I> Sub for Wrap<I> where I:Source<Out=Pair<<I as Other>::Item>>+Other{}
fn main(){}
