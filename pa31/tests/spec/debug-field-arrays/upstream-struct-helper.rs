    /// Shrinks `derive(Debug)` code, for faster compilation and smaller binaries.
    /// For the cases not covered by `debug_struct_field[12345]_finish`.
    #[doc(hidden)]
    #[unstable(feature = "fmt_helpers_for_derive", issue = "none")]
    pub fn debug_struct_fields_finish<'b>(
        &'b mut self,
        name: &str,
        names: &[&str],
        values: &[&dyn Debug],
    ) -> Result {
        assert_eq!(names.len(), values.len());
        let mut builder = builders::debug_struct_new(self, name);
        for (name, value) in iter::zip(names, values) {
            builder.field(name, value);
        }
        builder.finish()
    }

