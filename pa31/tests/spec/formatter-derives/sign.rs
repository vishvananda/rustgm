/// The signedness of a [`Formatter`] (or of a [`FormattingOptions`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[unstable(feature = "formatting_options", issue = "118117")]
pub enum Sign {
    /// Represents the `+` flag.
    Plus,
    /// Represents the `-` flag.
    Minus,
}
