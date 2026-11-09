trait Searcher<'a> {}
trait Keyed<'a, K>: Searcher<'a> {}
trait DoubleKeyed<'a, K>: Keyed<'a, K> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
    type Key;
}
trait Walk {}
trait BackWalk: Walk {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
impl<'a, P> Walk for Split<'a, P> where P: Pattern, P::Searcher<'a>: Keyed<'a, P::Key> {}
impl<'a, P> BackWalk for Split<'a, P> where P: Pattern, P::Searcher<'a>: DoubleKeyed<'a, P::Key> {}

fn main() {}
