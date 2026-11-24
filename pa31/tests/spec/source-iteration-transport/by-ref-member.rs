    // Reduced from the pinned core/src/iter/traits/iterator.rs.
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }

