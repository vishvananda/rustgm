#[rustc_on_unimplemented(
    on(
        any(Self = "std::path::Path", Self = "std::path::PathBuf"),
        label = "`{Self}` cannot be formatted with the default formatter; call `.display()` on it",
        note = "call `.display()` or `.to_string_lossy()` to safely print paths, \
                as they may contain non-Unicode data",
    ),
    on(
        from_desugaring = "FormatLiteral",
        note = "in format strings you may be able to use `{{:?}}` (or {{:#?}} for pretty-print) instead",
        label = "`{Self}` cannot be formatted with the default formatter",
    ),
    message = "`{Self}` doesn't implement `{This}`"
)]
#[doc(alias = "{}")]
#[rustc_diagnostic_item = "Display"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Display: PointeeSized {
    #[doc = include_str!("fmt_trait_method_doc.md")]
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fmt;
    ///
    /// struct Position {
    ///     longitude: f32,
    ///     latitude: f32,
    /// }
    ///
    /// impl fmt::Display for Position {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "({}, {})", self.longitude, self.latitude)
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     "(1.987, 2.983)",
    ///     format!("{}", Position { longitude: 1.987, latitude: 2.983, }),
    /// );
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}
