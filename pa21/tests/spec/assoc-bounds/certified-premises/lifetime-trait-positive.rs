trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait Pattern<'a>: Sized {
    type Searcher: Searcher<'a>;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<'a, P: Pattern<'a>>(P::Searcher);
impl<'a, P> Walk for Split<'a, P> where P: Pattern<'a, Searcher: ReverseSearcher<'a>> {}
impl<'a, P> FusedWalk for Split<'a, P> where P: Pattern<'a, Searcher: ReverseSearcher<'a>> {}

fn main() {}
