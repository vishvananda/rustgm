trait Super{}
trait Sub:Super{}
trait Source{type Out;}
impl<I,J,U> Super for (I,J) where I:Source<Out=U>,J:Source<Out=U>{}
impl<I:Source> Sub for (I,I){}
fn main(){}
