trait Lend<'a>{type Out;}
impl<'a>Lend<'a> for u8{type Out=u16;}
trait Apply{type Out;}
struct W<T>(T);
impl<T,U>Apply for W<T> where for<'a> T:Lend<'a,Out=U>{type Out=U;}
fn main(){}
