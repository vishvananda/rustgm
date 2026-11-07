trait Lend<'a>{type Out;}
trait Apply{type Out;}
struct W<T>(T);
impl<T,V,U>Apply for W<T> where for<'a> V:Lend<'a,Out=U>{type Out=(T,U);}
fn main(){}
