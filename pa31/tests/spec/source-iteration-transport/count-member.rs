    // Reduced from the pinned core/src/iter/traits/iterator.rs.
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized + [const] Destruct,
        F: [const] FnMut(B, Self::Item) -> B + [const] Destruct,
    {
        let mut accum = init;
        while let Maybe::Present(x) = self.next() {
            accum = f(accum, x);
        }
        accum
    }

    fn count(self) -> usize
    where
        Self: Sized + [const] Destruct,
        Self::Item: [const] Destruct,
    {
        self.fold(
            0,
            #[rustc_inherit_overflow_checks]
            const |accum, _elem| accum + 1,
        )
    }
