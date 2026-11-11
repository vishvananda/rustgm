trait Iter{type Item;}struct Holder<I:Iter>{iter:I}impl<I:Iter>Holder<I>where I::Item:Clone{fn get(&self)->u8{23}}
struct Known;impl Iter for Known{type Item=u8;}fn probe(h:&Holder<Known>)->u8{h.get()}
fn main(){}
