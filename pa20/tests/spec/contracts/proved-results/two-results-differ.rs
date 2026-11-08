trait Super{}
trait Sub:Super{}
trait Source{type Out;}
impl<I,J,U> Super for (I,J) where I:Source<Out=U>,J:Source<Out=U>{}
impl<I:Source,J:Source> Sub for (I,J){}
fn main(){}
