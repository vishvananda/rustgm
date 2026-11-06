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

