fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;}
trait Pick<Rhs>{fn pick(&self)->u8;}
struct W<X>(X);
impl<A:Pick<B>,B> Pick<W<B>> for W<A>{fn pick(&self)->u8{self.0.pick()+1}}
struct Num(u8);impl Pick<u8> for Num{fn pick(&self)->u8{self.0}}
struct Src;impl It for Src{type Item=Num;}
trait Marker{}impl<X:Marker> Pick<X> for X{fn pick(&self)->u8{50}}struct Mark;impl Marker for Mark{}
fn code<I:It,T>(w:W<I::Item>)->u8{<W<I::Item> as Pick<W<T>>>::pick(&w)}
fn main(){}
