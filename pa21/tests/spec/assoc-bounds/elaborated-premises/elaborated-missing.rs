trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait DoubleEndedSearcher<'a>: ReverseSearcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Walk {}
trait BackWalk: Walk {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
impl<'a, P> Walk for Split<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {}
impl<'a, P> BackWalk for Split<'a, P> where P: Pattern<Searcher<'a>: Searcher<'a>> {}

fn main() {}
