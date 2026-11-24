    // Reduced from the pinned core/src/iter/traits/iterator.rs.
    #[inline]
    fn size_hint(&self) -> (usize, Maybe<usize>) { (0, Maybe::Absent) }

