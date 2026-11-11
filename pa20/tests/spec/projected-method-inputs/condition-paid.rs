trait Iter{type Item;}struct Holder<I:Iter>{iter:I}impl<I:Iter>Holder<I>where I::Item:Clone{fn get(&self)->u8{23}}
fn probe<I:Iter>(h:&Holder<I>)->u8 where I::Item:Clone{h.get()}
fn main(){}
