#[derive(Copy, Debug, Hash)]
#[derive_const(Clone, Eq, PartialOrd, Ord, PartialEq)]
#[stable(feature = "rust1", since = "1.0.0")]
// This is a lang item only so that `BinOp::Cmp` in MIR can return it.
// It has no special behavior, but does require that the three variants
// `Less`/`Equal`/`Greater` remain `-1_i8`/`0_i8`/`+1_i8` respectively.
#[lang = "Ordering"]
#[repr(i8)]
pub enum Ordering {
    /// An ordering where a compared value is less than another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Less = -1,
    /// An ordering where a compared value is equal to another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Equal = 0,
    /// An ordering where a compared value is greater than another.
    #[stable(feature = "rust1", since = "1.0.0")]
    Greater = 1,
}
