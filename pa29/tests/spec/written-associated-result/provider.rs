pub trait Searcher<'a> {}
pub trait ReverseSearcher<'a>: Searcher<'a> {}
pub trait DoubleEndedSearcher<'a>: ReverseSearcher<'a> {}

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

pub trait Fused: Read {}

impl<'a, P> Fused for Cursor<'a, P> where P: Pattern<Searcher<'a>: ReverseSearcher<'a>> {}

pub trait DoubleEnded: Read {
    fn previous(&self) -> Self::Item;
}

impl<'a, P> DoubleEnded for Cursor<'a, P>
where
    P: Pattern<Searcher<'a>: DoubleEndedSearcher<'a>>,
{
    fn previous(&self) -> &'a u8 {
        self.0
    }
}

impl<'a> Searcher<'a> for &'a u8 {}
impl<'a> ReverseSearcher<'a> for &'a u8 {}
impl<'a> DoubleEndedSearcher<'a> for &'a u8 {}

impl Pattern for u8 {
    type Searcher<'a> = &'a u8;
}

impl<'a> Searcher<'a> for bool {}

impl Pattern for bool {
    type Searcher<'a> = bool;
}
