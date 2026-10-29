macro_rules! impl_zeroable_primitive {
    ($($NonZeroInner:ident ( $primitive:ty )),+ $(,)?) => {
        $(
            #[unstable(
                feature = "nonzero_internals",
                reason = "implementation detail which may disappear or be replaced at any time",
                issue = "none"
            )]
            unsafe impl ZeroablePrimitive for $primitive {
                type NonZeroInner = super::niche_types::$NonZeroInner;
            }
        )+
    };
}

