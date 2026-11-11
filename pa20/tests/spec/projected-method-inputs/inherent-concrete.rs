trait Family{type Out;}
struct Byte;impl Family for Byte{type Out=u8;}
struct Non;struct Other;impl Family for Other{type Out=Non;}
struct Hold<I:Family>{i:I}
impl<I:Family>Hold<I>where I::Out:Copy{fn choose(&self)->u8{23}}
trait Fallback{fn choose(&self)->u8;}
impl<I:Family>Fallback for Hold<I>{fn choose(&self)->u8{41}}
fn probe(h:&Hold<Byte>)->u8{h.choose()}fn main(){}
