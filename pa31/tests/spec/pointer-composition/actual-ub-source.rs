#[macro_export]
#[unstable(feature = "ub_checks", issue = "none")]
macro_rules! assert_unsafe_precondition {
    ($kind:ident, $message:expr, ($($name:ident:$ty:ty = $arg:expr),*$(,)?) => $e:expr $(,)?) => {
        {
            // This check is inlineable, but not by the MIR inliner.
            // The reason for this is that the MIR inliner is in an exceptionally bad position
            // to think about whether or not to inline this. In MIR, this call is gated behind `debug_assertions`,
            // which will codegen to `false` in release builds. Inlining the check would be wasted work in that case and
            // would be bad for compile times.
            //
            // LLVM on the other hand sees the constant branch, so if it's `false`, it can immediately delete it without
            // inlining the check. If it's `true`, it can inline it and get significantly better performance.
            #[rustc_no_mir_inline]
            #[inline]
            #[rustc_nounwind]
            #[track_caller]
            const fn precondition_check($($name:$ty),*) {
                if !$e {
                    let msg = concat!("unsafe precondition(s) violated: ", $message,
                        "\n\nThis indicates a bug in the program. \
                        This Undefined Behavior check is optional, and cannot be relied on for safety.");
                    ::core::panicking::panic_nounwind_fmt(::core::fmt::Arguments::from_str(msg), false);
                }
            }

            if ::core::ub_checks::$kind() {
                precondition_check($($arg,)*);
            }
        }
    };
}
#[unstable(feature = "ub_checks", issue = "none")]
pub use assert_unsafe_precondition;
/// Checking library UB is always enabled when UB-checking is done
/// (and we use a reexport so that there is no unnecessary wrapper function).
#[unstable(feature = "ub_checks", issue = "none")]
pub use intrinsics::ub_checks as check_library_ub;

/// Determines whether we should check for language UB.
///
/// The intention is to not do that when running in the interpreter, as that one has its own
/// language UB checks which generally produce better errors.
#[inline]
#[rustc_allow_const_fn_unstable(const_eval_select)]
pub(crate) const fn check_language_ub() -> bool {
    // Only used for UB checks so we may const_eval_select.
    const_eval_select!(
        @capture { } -> bool:
        if const {
            // Always disable UB checks.
            false
        } else {
            // Disable UB checks in Miri.
            !cfg!(miri)
        }
    ) && intrinsics::ub_checks()
}

/// Checks whether `ptr` is properly aligned with respect to the given alignment, and
/// if `is_zst == false`, that `ptr` is not null.
///
/// In `const` this is approximate and can fail spuriously. It is primarily intended
/// for `assert_unsafe_precondition!` with `check_language_ub`, in which case the
/// check is anyway not executed in `const`.
#[inline]
#[rustc_allow_const_fn_unstable(const_eval_select)]
pub(crate) const fn maybe_is_aligned_and_not_null(
    ptr: *const (),
    align: usize,
    is_zst: bool,
) -> bool {
    // This is just for safety checks so we can const_eval_select.
    maybe_is_aligned(ptr, align) && (is_zst || !ptr.is_null())
}

/// Checks whether `ptr` is properly aligned with respect to the given alignment.
///
/// In `const` this is approximate and can fail spuriously. It is primarily intended
/// for `assert_unsafe_precondition!` with `check_language_ub`, in which case the
/// check is anyway not executed in `const`.
#[inline]
#[rustc_allow_const_fn_unstable(const_eval_select)]
pub(crate) const fn maybe_is_aligned(ptr: *const (), align: usize) -> bool {
    // This is just for safety checks so we can const_eval_select.
    const_eval_select!(
        @capture { ptr: *const (), align: usize } -> bool:
        if const {
            true
        } else {
            ptr.is_aligned_to(align)
        }
    )
}

#[inline]
pub(crate) const fn is_valid_allocation_size(size: usize, len: usize) -> bool {
    let max_len = if size == 0 { usize::MAX } else { isize::MAX as usize / size };
    len <= max_len
}
