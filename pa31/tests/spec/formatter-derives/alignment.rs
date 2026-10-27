#[stable(feature = "fmt_flags_align", since = "1.28.0")]
#[rustc_diagnostic_item = "Alignment"]
/// Possible alignments returned by `Formatter::align`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alignment {
    #[stable(feature = "fmt_flags_align", since = "1.28.0")]
    /// Indication that contents should be left-aligned.
    Left,
    #[stable(feature = "fmt_flags_align", since = "1.28.0")]
    /// Indication that contents should be right-aligned.
    Right,
    #[stable(feature = "fmt_flags_align", since = "1.28.0")]
    /// Indication that contents should be center-aligned.
    Center,
}
