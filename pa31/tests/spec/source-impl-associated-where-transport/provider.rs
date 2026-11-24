pub trait Search<'a> {}

pub trait Pattern {
    type Searcher<'a>: Search<'a>
    where
        Self: 'a;
}

pub struct Pair<'a, 'b>(pub &'a u8, pub &'b u8);
pub struct Needle<'b>(pub &'b u8);

impl<'a, 'b> Search<'a> for Pair<'a, 'b> {}

impl<'b> Pattern for Needle<'b> {
    type Searcher<'a> = Pair<'a, 'b>
    where
        Self: 'a;
}
