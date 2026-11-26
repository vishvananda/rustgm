#[stable(feature = "stable_constant", since = "1.0.0")]
pub const STABLE: u8 = 42;

#[unstable(feature = "experimental_constant", issue = "none")]
pub const EXPERIMENTAL: u8 = 57;

#[stable(feature = "nested_constant", since = "1.0.0")]
pub mod nested {
    #[stable(feature = "nested_constant", since = "1.0.0")]
    pub const VALUE: u8 = 83;
}
