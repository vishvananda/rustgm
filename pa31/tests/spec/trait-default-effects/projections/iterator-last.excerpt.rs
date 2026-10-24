    fn last(self) -> Option<Self::Item>
    where
        Self: Sized + [const] Destruct,
        Self::Item: [const] Destruct,
    {
        #[inline]
        #[rustc_const_unstable(feature = "const_destruct", issue = "133214")]
        const fn some<T>(_: Option<T>, x: T) -> Option<T>
        where
            T: [const] Destruct,
        {
            Some(x)
        }

        self.fold(None, some)
    }
