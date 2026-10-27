// Declaration/expansion reducer only; unresolved formatter/Arguments names are
// intentional at this stage. Runtime uses must retain their real source bodies.
// Rust88d9 core fmt/mod.rs, fmt/rt.rs and panic/panic_info.rs selected declarations.
pub mod fmt {
    use crate::{derive, NonNull, PhantomData, PointeeSized};
    use crate::marker::Copy;
    use crate::clone::Clone;

    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "Debug"]
    #[rustc_trivial_field_reads]
    pub trait Debug: PointeeSized {
        #[stable(feature = "rust1", since = "1.0.0")]
        fn fmt(&self, f: &mut Formatter<'_>) -> Result;
    }
    pub(crate) mod macros {
        #[rustc_builtin_macro]
        #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
        #[allow_internal_unstable(core_intrinsics, fmt_helpers_for_derive)]
        pub macro Debug($item:item) { /* compiler built-in */ }
    }
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    pub use macros::Debug;

    #[derive(Copy, Clone)]
    enum ArgumentType<'a> {
        Placeholder {
            value: NonNull<()>,
            formatter: unsafe fn(NonNull<()>, &mut Formatter<'_>) -> Result,
            _lifetime: PhantomData<&'a ()>,
        },
        Count(u16),
    }
}
pub mod panic_info {
    use crate::{derive, fmt};
    use crate::fmt::Debug;
    use crate::location::Location;
    #[lang = "panic_info"]
    #[stable(feature = "panic_hooks", since = "1.10.0")]
    #[derive(Debug)]
    pub struct PanicInfo<'a> {
        message: &'a fmt::Arguments<'a>,
        location: &'static Location<'static>,
        can_unwind: bool,
        force_no_backtrace: bool,
    }
}
