//! Nominal library identities are independent of user declaration spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LibraryType {
    Option,
    Result,
    ManuallyDrop,
}
impl LibraryType {
    pub const ALL: [Self; 3] = [Self::Option, Self::Result, Self::ManuallyDrop];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Option => "Option",
            Self::Result => "Result",
            Self::ManuallyDrop => "ManuallyDrop",
        }
    }
    #[must_use]
    pub const fn module(self) -> &'static str {
        match self {
            Self::Option => "option",
            Self::Result => "result",
            Self::ManuallyDrop => "mem",
        }
    }
    #[must_use]
    pub const fn parameters(self) -> &'static [&'static str] {
        match self {
            Self::Option | Self::ManuallyDrop => &["T"],
            Self::Result => &["T", "E"],
        }
    }
    #[must_use]
    pub const fn variants(self) -> &'static [(&'static str, Option<u32>)] {
        match self {
            Self::Option => &[("None", None), ("Some", Some(0))],
            Self::Result => &[("Ok", Some(0)), ("Err", Some(1))],
            Self::ManuallyDrop => &[("new", Some(0))],
        }
    }
}
