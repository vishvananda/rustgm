trait Searcher<'a> {}
trait ReverseSearcher<'a>: Searcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Walk {}
trait FusedWalk: Walk {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
fn need<'a, P>() where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {}
fn call<'a, P>() where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {
    need::<'a, P>();
}

fn main() {}
