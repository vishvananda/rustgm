#[lang = "RangeInclusive"]
pub struct RangeInclusive<Idx> {
    // Note that the fields here are not public to allow changing the
    // representation in the future; in particular, while we could plausibly
    // expose start/end, modifying them without changing (future/current)
    // private fields may lead to incorrect behavior, so we don't want to
    // support that mode.
    pub(crate) start: Idx,
    pub(crate) end: Idx,

    // This field is:
    //  - `false` upon construction
    //  - `false` when iteration has yielded an element and the iterator is not exhausted
    //  - `true` when iteration has been used to exhaust the iterator
    //
    // This is required to support PartialEq and Hash without a PartialOrd bound or specialization.
    pub(crate) exhausted: bool,
}
