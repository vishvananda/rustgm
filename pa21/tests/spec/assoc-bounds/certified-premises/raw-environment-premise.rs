trait Searcher<'a> {}
trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}
trait Tr {
    type Out;
}
trait Marker {}
struct Split<'a, P: Pattern>(P::Searcher<'a>);
trait Walk {}
impl<'a, P> Walk for Split<'a, P>
where
    P: Pattern,
    for<'x> P::Searcher<'x>: Tr,
    <P::Searcher<'a> as Tr>::Out: Marker,
{
}

fn main() {}
