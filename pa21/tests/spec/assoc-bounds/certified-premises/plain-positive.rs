trait Searcher {}
trait ReverseSearcher: Searcher {}
trait Pattern: Sized {
    type Searcher: Searcher;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<P: Pattern>(P::Searcher);
impl<P> Walk for Split<P> where P: Pattern<Searcher: ReverseSearcher> {}
impl<P> FusedWalk for Split<P> where P: Pattern<Searcher: ReverseSearcher> {}

fn main() {}
