fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Seq{type Item;fn next(&mut self)->Option<Self::Item>;
fn map<B,F:FnMut(Self::Item)->B>(self,f:F)->Mapped<Self,F> where Self:Sized{Mapped{seq:self,f}}}
struct Mapped<S,F>{seq:S,f:F}
impl<B,S:Seq,F:FnMut(S::Item)->B> Seq for Mapped<S,F>{type Item=B;fn next(&mut self)->Option<B>{match self.seq.next(){Some(i)=>Some((self.f)(i)),None=>None}}}
struct Iter<'a>{items:&'a [Word],at:usize}
impl<'a> Seq for Iter<'a>{type Item=&'a Word;fn next(&mut self)->Option<&'a Word>{if self.at<self.items.len(){self.at+=1;Some(&self.items[self.at-1])}else{None}}}
struct Text(u8);
struct Word(Text);
impl Word{fn text(&self)->&Text{&self.0}}
trait View<T>{fn view(&self)->&T;}
impl View<Text> for Word{fn view(&self)->&Text{&self.0}}
fn total<'a>(mut s:impl Seq<Item=&'a Text>)->u8{let mut t=0;while let Some(x)=s.next(){t+=x.0;}t}
fn main(){let w=[Word(Text(2)),Word(Text(3))];check(total(Iter{items:&w,at:0}.map(Word::text))==5);}
