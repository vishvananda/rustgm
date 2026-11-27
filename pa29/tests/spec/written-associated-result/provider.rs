pub trait Searcher<'a> {}
pub trait ReverseSearcher<'a>: Searcher<'a> {}

pub trait Pattern: Sized {
    type Searcher<'a>: Searcher<'a>;
}

pub struct Cursor<'a, P: Pattern>(pub &'a u8, pub P);

pub trait Read {
    type Item;

    fn read(&self) -> Self::Item;
}

impl<'a, P> Read for Cursor<'a, P>
where
    P: Pattern<Searcher<'a>: ReverseSearcher<'a>>,
{
    type Item = &'a u8;

    fn read(&self) -> Self::Item {
        self.0
    }
}

impl<'a> Searcher<'a> for &'a u8 {}
impl<'a> ReverseSearcher<'a> for &'a u8 {}

impl Pattern for u8 {
    type Searcher<'a> = &'a u8;
}

impl<'a> Searcher<'a> for bool {}

impl Pattern for bool {
    type Searcher<'a> = bool;
}
