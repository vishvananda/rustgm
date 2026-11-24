    // Reduced from the pinned core/src/iter/traits/iterator.rs; core's own
    // non-capturing `const |accum, _elem| accum + 1` callback remains separate
    // closure-body transport work, so this fixture exercises the count
    // reduction header and const Add companion with a nested const fn helper.
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
        #[inline]
        const fn add_one<T>(accum: usize, _: T) -> usize
        where
            T: [const] Destruct,
        {
            accum + 1
        }

        self.fold(0, add_one)
    }
