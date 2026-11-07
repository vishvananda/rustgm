trait Deref { type Target; }
trait DerefMut: Deref {}
struct Pin<P>(P);
impl<P: Deref> Deref for Pin<P> { type Target = P::Target; }
impl<P: Deref> DerefMut for Pin<P> where Self::Target: Copy {}
fn main() {}
