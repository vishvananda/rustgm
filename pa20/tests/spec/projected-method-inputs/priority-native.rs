fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Family{type Out;}
struct Byte;impl Family for Byte{type Out=u8;}
struct Non;struct Other;impl Family for Other{type Out=Non;}
struct Hold<I:Family>{i:I}
impl<I:Family>Hold<I>where I::Out:Copy{fn choose(&self)->u8{23}}
trait Fallback{fn choose(&self)->u8;}
impl<I:Family>Fallback for Hold<I>{fn choose(&self)->u8{41}}

fn unpaid<I:Family>(h:&Hold<I>)->u8{h.choose()}
fn paid<I:Family>(h:&Hold<I>)->u8 where I::Out:Copy{h.choose()}
trait Output{type Value;}impl Output for Byte{type Value=u8;}
#[derive(Clone,Copy)]struct Wrap<T>(T);
impl<T>Wrap<T>{fn get(&self)->u8{31}}
fn projected<I:Output>(w:&Wrap<I::Value>)->u8{w.get()}
fn main(){
let a=Hold{i:Byte};let b=Hold{i:Other};
check(unpaid(&a)==41);check(unpaid(&b)==41);
check(paid(&a)==23);check(a.choose()==23);check(b.choose()==41);
check(projected::<Byte>(&Wrap(7))==31);
}
