fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait It{type Item;}
trait Pick<Rhs>{fn pick(&self)->u8;}
struct W<X>(X);
impl<A:Pick<B>,B> Pick<W<B>> for W<A>{fn pick(&self)->u8{self.0.pick()+1}}
struct Num(u8);impl Pick<u8> for Num{fn pick(&self)->u8{self.0}}
struct Src;impl It for Src{type Item=Num;}
impl Pick<W<u8>> for W<u16>{fn pick(&self)->u8{100}}
fn code<I:It>(w:W<I::Item>)->u8 where I::Item:Pick<u8>{<W<I::Item> as Pick<W<u8>>>::pick(&w)}
fn main(){check(code::<Src>(W(Num(4)))==5);check(<W<u16> as Pick<W<u8>>>::pick(&W(3))==100);}
