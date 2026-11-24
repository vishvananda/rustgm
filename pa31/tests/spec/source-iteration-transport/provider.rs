// Authored reduction of pinned core/iter/traits/{iterator,collect}.rs.
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}
impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}

#[lang="Option"] pub enum Maybe<T> {
    #[lang="Some"] Present(T),
    #[lang="None"] Absent,
}

#[lang="iterator"] pub const trait Iterator {
    type Item;
    #[lang="next"] fn next(&mut self) -> Maybe<Self::Item>;
}

#[rustc_skip_during_method_dispatch(array, boxed_slice)]
pub const trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item=Self::Item>;
    #[lang="into_iter"] fn into_iter(self) -> Self::IntoIter;
}
