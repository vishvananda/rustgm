#[derive(Copy, PartialOrd, Ord)]
#[derive_const(Clone, Eq)]
#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "type_id"]
pub struct TypeId {
    /// This needs to be an array of pointers, since there is provenance
    /// in the first array field. This provenance knows exactly which type
    /// the TypeId actually is, allowing CTFE and miri to operate based off it.
    /// At runtime all the pointers in the array contain bits of the hash, making
    /// the entire `TypeId` actually just be a `u128` hash of the type.
    pub(crate) data: [*const (); 16 / size_of::<*const ()>()],
}
