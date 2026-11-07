trait Deref { type Target; }
trait DerefMut: Deref {}
trait Helper { type Target; }
struct Pin<P>(P);
struct PinHelper<P>(P);
impl<P: Deref> Deref for Pin<P> { type Target = P::Target; }
impl<P> DerefMut for Pin<P> where P: Deref, PinHelper<P>: Helper<Target = Self::Target> {}
fn main() {}
