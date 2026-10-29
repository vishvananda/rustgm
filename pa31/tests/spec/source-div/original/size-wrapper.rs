#[inline(always)]
#[must_use]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_promotable]
#[rustc_const_stable(feature = "const_mem_size_of", since = "1.24.0")]
#[rustc_diagnostic_item = "mem_size_of"]
pub const fn size_of<T>() -> usize {
    // By making this a constant, we also guarantee that the constant can be successfully evaluated
    // in any program execution that actually executes `size_of`. Which is relevant because the
    // constant can fail to evaluate if the type is too big. Someone might do something cursed where
    // soundness relies on a certain type not being too big, and they check that by just invoking
    // size_of on the type to ensure it exists, so if we fully DCE'd size_of calls that would be
    // considered unsound... but by making this a constant, it participates in the usual "required
    // consts" system, and we are safe.
    <T as SizedTypeProperties>::SIZE
}
