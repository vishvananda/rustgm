/// Options for formatting.
///
/// `FormattingOptions` is a [`Formatter`] without an attached [`Write`] trait.
/// It is mainly used to construct `Formatter` instances.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[unstable(feature = "formatting_options", issue = "118117")]
pub struct FormattingOptions {
    /// Flags, with the following bit fields:
    ///
    /// ```text
    ///   31  30  29  28  27  26  25  24  23  22  21  20                              0
    /// ┌───┬───────┬───┬───┬───┬───┬───┬───┬───┬───┬──────────────────────────────────┐
    /// │ 0 │ align │ p │ w │ X?│ x?│'0'│ # │ - │ + │               fill               │
    /// └───┴───────┴───┴───┴───┴───┴───┴───┴───┴───┴──────────────────────────────────┘
    ///   │     │     │   │  └─┬───────────────────┘ └─┬──────────────────────────────┘
    ///   │     │     │   │    │                       └─ The fill character (21 bits char).
    ///   │     │     │   │    └─ The debug upper/lower hex, zero pad, alternate, and plus/minus flags.
    ///   │     │     │   └─ Whether a width is set. (The value is stored separately.)
    ///   │     │     └─ Whether a precision is set. (The value is stored separately.)
    ///   │     ├─ 0: Align left. (<)
    ///   │     ├─ 1: Align right. (>)
    ///   │     ├─ 2: Align center. (^)
    ///   │     └─ 3: Alignment not set. (default)
    ///   └─ Always zero.
    /// ```
    // Note: This could use a pattern type with range 0x0000_0000..=0x7dd0ffff.
    // It's unclear if that's useful, though.
    flags: u32,
    /// Width if width flag (bit 27) above is set. Otherwise, always 0.
    width: u16,
    /// Precision if precision flag (bit 28) above is set. Otherwise, always 0.
    precision: u16,
}
