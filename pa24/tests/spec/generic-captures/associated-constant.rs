fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct L;impl L{const MAX:u64=18446744073709551615;}
trait Drain{const STEP:u64;fn drain_to(&mut self,limit:u64)->u64;fn peek(&self,limit:u64)->u64;}
impl Drain for u8{const STEP:u64=3;fn drain_to(&mut self,limit:u64)->u64{*self+=1;if limit==L::MAX{*self as u64}else{limit}}fn peek(&self,limit:u64)->u64{if limit==L::MAX{*self as u64}else{0}}}
fn drain<R:Drain>(r:&mut R)->u64{let mut flush=||->u64{r.drain_to(L::MAX)};flush()+flush()}
fn step<R:Drain>(r:&mut R)->u64{let mut flush=||->u64{r.drain_to(R::STEP)};flush()}
fn look<R:Drain>(r:&R)->u64{let peek=||->u64{r.peek(L::MAX)};peek()+peek()}
fn main(){let mut x=0u8;check(drain(&mut x)==3&&x==2);check(step(&mut x)==3&&x==3);check(look(&x)==6);}
