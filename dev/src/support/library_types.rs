//! Nominal library identities are independent of user declaration spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryValue {
    EnvironmentOption,
}
impl LibraryValue {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EnvironmentOption => "option-environment",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LibraryType {
    Option,
    Result,
    ManuallyDrop,
    AtomicU32,
    Ordering,
}
#[derive(Clone, Copy)]
pub enum LibraryField {
    Parameter(u32),
    U32,
}
#[derive(Clone, Copy)]
pub enum LibraryModule {
    Option,
    Result,
    Memory,
    Sync,
    Atomic,
}
impl LibraryModule {
    pub const COUNT: usize = 5;
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Option => "option",
            Self::Result => "result",
            Self::Memory => "mem",
            Self::Sync => "sync",
            Self::Atomic => "atomic",
        }
    }
}
impl LibraryType {
    pub const ALL: [Self; 5] = [
        Self::Option,
        Self::Result,
        Self::ManuallyDrop,
        Self::AtomicU32,
        Self::Ordering,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Option => "Option",
            Self::Result => "Result",
            Self::ManuallyDrop => "ManuallyDrop",
            Self::AtomicU32 => "AtomicU32",
            Self::Ordering => "Ordering",
        }
    }
    #[must_use]
    pub const fn module(self) -> &'static str {
        match self {
            Self::Option => "option",
            Self::Result => "result",
            Self::ManuallyDrop => "mem",
            Self::AtomicU32 | Self::Ordering => "atomic",
        }
    }
    #[must_use]
    pub const fn modules(self) -> &'static [LibraryModule] {
        match self {
            Self::Option => &[LibraryModule::Option],
            Self::Result => &[LibraryModule::Result],
            Self::ManuallyDrop => &[LibraryModule::Memory],
            Self::AtomicU32 | Self::Ordering => &[LibraryModule::Sync, LibraryModule::Atomic],
        }
    }
    #[must_use]
    pub const fn is_struct(self) -> bool {
        matches!(self, Self::ManuallyDrop | Self::AtomicU32)
    }
    #[must_use]
    pub const fn in_prelude(self) -> bool {
        matches!(self, Self::Option | Self::Result)
    }
    #[must_use]
    pub const fn parameters(self) -> &'static [&'static str] {
        match self {
            Self::Option | Self::ManuallyDrop => &["T"],
            Self::Result => &["T", "E"],
            Self::AtomicU32 | Self::Ordering => &[],
        }
    }
    #[must_use]
    pub const fn variants(self) -> &'static [(&'static str, Option<LibraryField>)] {
        match self {
            Self::Option => &[("None", None), ("Some", Some(LibraryField::Parameter(0)))],
            Self::Result => &[
                ("Ok", Some(LibraryField::Parameter(0))),
                ("Err", Some(LibraryField::Parameter(1))),
            ],
            Self::ManuallyDrop => &[("new", Some(LibraryField::Parameter(0)))],
            Self::AtomicU32 => &[("new", Some(LibraryField::U32))],
            Self::Ordering => &[
                ("Relaxed", None),
                ("Release", None),
                ("Acquire", None),
                ("AcqRel", None),
                ("SeqCst", None),
            ],
        }
    }
}
