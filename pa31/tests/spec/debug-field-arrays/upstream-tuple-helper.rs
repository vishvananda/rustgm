    /// Shrinks `derive(Debug)` code, for faster compilation and smaller
    /// binaries. For the cases not covered by `debug_tuple_field[12345]_finish`.
    #[doc(hidden)]
    #[unstable(feature = "fmt_helpers_for_derive", issue = "none")]
    pub fn debug_tuple_fields_finish<'b>(
        &'b mut self,
        name: &str,
        values: &[&dyn Debug],
    ) -> Result {
        let mut builder = builders::debug_tuple_new(self, name);
        for value in values {
            builder.field(value);
        }
        builder.finish()
    }

