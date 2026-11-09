trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait DoubleEndedSearcher<'a>: ReverseSearcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
enum Opt<T> {
    Some(T),
    None,
}
trait Iter {
    type Item;
    fn next(&mut self) -> Opt<Self::Item>;
}
trait DoubleIter: Iter {
    fn next_back(&mut self) -> Opt<Self::Item>;
}
struct Internal<'a, P: Pattern> {
    matcher: P::Searcher<'a>,
}
struct RSplit<'a, P: Pattern>(Internal<'a, P>);
impl<'a, P> Iter for RSplit<'a, P>
where
    P: Pattern,
    P::Searcher<'a>: ReverseSearcher<'a>,
{
    type Item = &'a u8;
    fn next(&mut self) -> Opt<&'a u8> {
        Opt::None
    }
}
impl<'a, P> DoubleIter for RSplit<'a, P>
where
    P: Pattern,
    P::Searcher<'a>: DoubleEndedSearcher<'a>,
{
    fn next_back(&mut self) -> Opt<&'a u8> {
        Opt::None
    }
}

fn main() {}
