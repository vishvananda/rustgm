// An adapter's field names its inner sequence through the nested projection
// `<I::Item as IntoSeq>::Seq`, as core's `Flatten<I>` holds
// `FlattenCompat<I, <I::Item as IntoIterator>::IntoIter>`. The inner
// instance's caller passes its front sequence at the lifetime that field
// normalization chose, which only the where clause `I::Item: IntoSeq<Seq = U>`
// relates to the item's lifetime; the callee's template names both at once.
fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Seq {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
struct Pair {
    a: u32,
    b: u32,
}
struct PairSeq<'a> {
    pair: &'a Pair,
    at: u8,
}
impl<'a> Seq for PairSeq<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<&'a u32> {
        self.at += 1;
        if self.at == 1 {
            Some(&self.pair.a)
        } else if self.at == 2 {
            Some(&self.pair.b)
        } else {
            None
        }
    }
}
trait IntoSeq {
    type Seq: Seq;
    fn into_seq(self) -> Self::Seq;
}
impl<'a> IntoSeq for &'a Pair {
    type Seq = PairSeq<'a>;
    fn into_seq(self) -> PairSeq<'a> {
        PairSeq { pair: self, at: 0 }
    }
}
struct Pairs<'a> {
    pairs: &'a [Pair; 2],
    at: usize,
}
impl<'a> Seq for Pairs<'a> {
    type Item = &'a Pair;
    fn next(&mut self) -> Option<&'a Pair> {
        if self.at < 2 {
            self.at += 1;
            Some(&self.pairs[self.at - 1])
        } else {
            None
        }
    }
}
// The front sequence is a parameter of its own, equal to the item's sequence
// only through the where clause, as in core's `FlattenCompat<I, U>`.
struct Flat<I, U> {
    outer: I,
    front: Option<U>,
}
fn refill<T: IntoSeq>(front: &mut Option<T::Seq>, item: T) {
    *front = Some(item.into_seq());
}
impl<I, U> Flat<I, U>
where
    I: Seq,
    I::Item: IntoSeq<Seq = U>,
    U: Seq,
{
    fn next(&mut self) -> Option<U::Item> {
        loop {
            if let Some(front) = &mut self.front {
                if let Some(x) = front.next() {
                    return Some(x);
                }
            }
            let item = match self.outer.next() {
            Some(value) => value,
            None => return None,
        };
            refill::<I::Item>(&mut self.front, item);
        }
    }
}
// The adapter names its inner type through the nested projection
// `<I::Item as IntoSeq>::Seq`, as core's `Flatten<I>` holds
// `FlattenCompat<I, <I::Item as IntoIterator>::IntoIter>`; its users select
// the inner instance from that normalized field.
struct Flatten<I: Seq>
where
    I::Item: IntoSeq,
{
    inner: Flat<I, <I::Item as IntoSeq>::Seq>,
}
impl<I: Seq> Flatten<I>
where
    I::Item: IntoSeq,
{
    fn new(outer: I) -> Self {
        Flatten {
            inner: Flat { outer, front: None },
        }
    }
    fn next(&mut self) -> Option<<<I::Item as IntoSeq>::Seq as Seq>::Item> {
        self.inner.next()
    }
}
struct Fuse<I> {
    iter: I,
}
impl<I: Seq> Seq for Fuse<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}
fn sum<'a, I: Seq<Item = &'a Pair>>(outer: I) -> u32 {
    let mut flat = Flatten::new(Fuse { iter: outer });
    let mut sum = 0;
    while let Some(x) = flat.next() {
        sum += *x;
    }
    sum
}
fn main() {
    let pairs = [Pair { a: 1, b: 2 }, Pair { a: 3, b: 4 }];
    check(sum(Pairs { pairs: &pairs, at: 0 }) == 10);
}
