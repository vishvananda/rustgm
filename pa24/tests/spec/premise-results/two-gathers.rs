fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make:Sized{fn make(seed:u8)->Option<Self>;}
impl Make for i64{fn make(seed:u8)->Option<i64>{Some(seed as i64*10)}}
impl Make for u8{fn make(seed:u8)->Option<u8>{Some(seed)}}
trait Gather<T>{fn gather<S:Source<Item=T>>(s:S)->Self;}
struct Total(i64);
impl Gather<i64> for Total{fn gather<S:Source<Item=i64>>(mut s:S)->Total{let mut t=0;while let Some(x)=s.next(){t+=x;}Total(t)}}
struct Counter(u8);
impl Source for Counter{type Item=u8;fn next(&mut self)->Option<u8>{if self.0==3{None}else{self.0+=1;Some(self.0)}}}
struct FilterMap<I,F>(I,F);
impl<B,I:Source,F:FnMut(I::Item)->Option<B>> Source for FilterMap<I,F>{type Item=B;fn next(&mut self)->Option<B>{while let Some(x)=self.0.next(){if let Some(y)=(self.1)(x){return Some(y);}}None}}
trait Source{type Item;fn next(&mut self)->Option<Self::Item>;}
impl Gather<u8> for Total{fn gather<S:Source<Item=u8>>(mut s:S)->Total{let mut t=0;while let Some(x)=s.next(){t+=x as i64;}Total(t)}}
fn main(){let t:Total=Gather::gather(FilterMap(Counter(0),|a|Make::make(a)));check(t.0==60);}
