// Reduced from the pinned core Deref and DerefMut declarations and reference impls.
pub mod ops {
    use crate::PointeeSized;

    #[lang = "deref"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_convert", issue = "143773")]
    pub const trait Deref: PointeeSized {
        #[lang = "deref_target"]
        #[stable(feature = "rust1", since = "1.0.0")]
        type Target: ?Sized;

        #[stable(feature = "rust1", since = "1.0.0")]
        fn deref(&self) -> &Self::Target;
    }

    #[lang = "deref_mut"]
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_convert", issue = "143773")]
    pub const trait DerefMut: [const] Deref + PointeeSized {
        #[stable(feature = "rust1", since = "1.0.0")]
        fn deref_mut(&mut self) -> &mut Self::Target;
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_convert", issue = "143773")]
    const impl<T: ?Sized> Deref for &T {
        type Target = T;

        fn deref(&self) -> &T { self }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_convert", issue = "143773")]
    const impl<T: ?Sized> Deref for &mut T {
        type Target = T;

        fn deref(&self) -> &T { self }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_const_unstable(feature = "const_convert", issue = "143773")]
    const impl<T: ?Sized> DerefMut for &mut T {
        fn deref_mut(&mut self) -> &mut T { self }
    }
}
