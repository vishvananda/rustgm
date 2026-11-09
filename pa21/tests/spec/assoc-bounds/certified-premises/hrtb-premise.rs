trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
impl<'a, P> Walk for Split<'a, P> where P: Pattern, P::Searcher<'a>: ReverseSearcher<'a> {}
impl<'a, P> FusedWalk for Split<'a, P> where P: Pattern, for<'x> P::Searcher<'x>: ReverseSearcher<'x> {}

fn main() {}
