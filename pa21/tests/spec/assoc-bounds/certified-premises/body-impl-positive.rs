trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
impl<'a, P> Walk for Split<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {}
fn need_walk<T: Walk>() {}
fn call<'a, P>() where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {
    need_walk::<Split<'a, P>>();
}

fn main() {}
