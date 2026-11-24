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

    fn last(self) -> Maybe<Self::Item>
    where
        Self: Sized + [const] Destruct,
        Self::Item: [const] Destruct,
    {
        #[inline]
        const fn some<T>(_: Maybe<T>, x: T) -> Maybe<T>
        where
            T: [const] Destruct,
        {
            Maybe::Present(x)
        }

        self.fold(Maybe::Absent, some)
    }
