trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<P: Pattern>(P::Searcher<'static>);
impl<P> Walk for Split<P> where P: Pattern<Searcher<'static>: ReverseSearcher<'static>> {}
impl<P> FusedWalk for Split<P> where P: Pattern<Searcher<'static>: ReverseSearcher<'static>> {}

fn main() {}
