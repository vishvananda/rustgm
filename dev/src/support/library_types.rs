//! Nominal library identities are independent of user declaration spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryValue {
    EnvironmentOption,
    /// Canonical protocol selector; actual owner and call permissions are checked later.
    FormatLiteral {
        constructor: crate::support::names::NameId,
    },
    /// The ordered special sites belong to this exact formatting invocation.
    FormatOutput {
        sites: FormatSites,
    },
    FormatConstructor {
        slot: u32,
        constructor: FormatConstructor,
        name: crate::support::names::NameId,
    },
    FormatSuperLet {
        slot: u32,
    },
    /// Installation replaces this draft-only occurrence with its parsed input.
    FormatInput {
        slot: u32,
    },
}
/// Coordinates in the producing syntax's shared edge arena; not a permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatSites {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatConstructor {
    Arguments,
    Debug,
}
impl FormatConstructor {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arguments => "new",
            Self::Debug => "new_debug",
        }
    }
    #[must_use]
    pub const fn owner(self) -> crate::support::bindings::LangItem {
        match self {
            Self::Arguments => crate::support::bindings::LangItem::FormatArguments,
            Self::Debug => crate::support::bindings::LangItem::FormatArgument,
        }
    }
}
impl LibraryValue {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EnvironmentOption => "option-environment",
            Self::FormatLiteral { .. } => "format-literal",
            Self::FormatOutput { .. } => "format-output",
            Self::FormatConstructor { .. } => "format-constructor",
            Self::FormatSuperLet { .. } => "format-super-let",
            Self::FormatInput { .. } => "format-input-draft",
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
    ComparisonOrdering,
    SliceIter,
    SliceIterMut,
    Range,
}
#[derive(Clone, Copy)]
pub enum LibraryParameter {
    Lifetime(&'static str),
    Type { name: &'static str, sized: bool },
}
impl LibraryParameter {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lifetime(name) | Self::Type { name, .. } => name,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryField {
    Parameter(u32),
    U32,
    SharedSlice,
    MutableSlice,
    RangeBounds,
}
#[derive(Clone, Copy)]
pub enum LibraryTag {
    U8,
    I8,
}
#[derive(Clone, Copy)]
pub enum LibraryModule {
    Option,
    Result,
    Memory,
    Sync,
    Atomic,
    Comparison,
    Slice,
}
impl LibraryModule {
    pub const COUNT: usize = 7;
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
            Self::Comparison => "cmp",
            Self::Slice => "slice",
        }
    }
}
impl LibraryType {
    pub const ALL: [Self; 9] = [
        Self::Option,
        Self::Result,
        Self::ManuallyDrop,
        Self::AtomicU32,
        Self::Ordering,
        Self::ComparisonOrdering,
        Self::SliceIter,
        Self::SliceIterMut,
        Self::Range,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Option => "Option",
            Self::Result => "Result",
            Self::ManuallyDrop => "ManuallyDrop",
            Self::AtomicU32 => "AtomicU32",
            Self::Ordering | Self::ComparisonOrdering => "Ordering",
            Self::SliceIter => "Iter",
            Self::SliceIterMut => "IterMut",
            Self::Range => "Range",
        }
    }
    /// Recognizes descriptor names for lazy preparation; lookup still proves identity.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Option" => Some(Self::Option),
            "Result" => Some(Self::Result),
            "ManuallyDrop" => Some(Self::ManuallyDrop),
            "AtomicU32" => Some(Self::AtomicU32),
            "Ordering" => Some(Self::Ordering),
            "Iter" => Some(Self::SliceIter),
            "IterMut" => Some(Self::SliceIterMut),
            "Range" => Some(Self::Range),
            _ => None,
        }
    }
    #[must_use]
    pub const fn module(self) -> &'static str {
        match self {
            Self::Option => "option",
            Self::Result => "result",
            Self::ManuallyDrop => "mem",
            Self::AtomicU32 | Self::Ordering => "atomic",
            Self::ComparisonOrdering => "cmp",
            Self::SliceIter | Self::SliceIterMut => "slice",
            Self::Range => "ops",
        }
    }
    #[must_use]
    pub const fn modules(self) -> &'static [LibraryModule] {
        match self {
            Self::Option => &[LibraryModule::Option],
            Self::Result => &[LibraryModule::Result],
            Self::ManuallyDrop => &[LibraryModule::Memory],
            Self::AtomicU32 | Self::Ordering => &[LibraryModule::Sync, LibraryModule::Atomic],
            Self::ComparisonOrdering => &[LibraryModule::Comparison],
            Self::SliceIter | Self::SliceIterMut => &[LibraryModule::Slice],
            Self::Range => &[],
        }
    }
    #[must_use]
    pub const fn tag(self) -> Option<LibraryTag> {
        match self {
            Self::Ordering => Some(LibraryTag::U8),
            Self::ComparisonOrdering => Some(LibraryTag::I8),
            _ => None,
        }
    }
    /// Representation-width bits, not a source ordinal or host signed integer.
    #[must_use]
    pub fn discriminant(self, ordinal: u32) -> Option<u128> {
        self.variants().get(usize::try_from(ordinal).ok()?)?;
        Some(if matches!(self, Self::ComparisonOrdering) {
            match ordinal {
                0 => 255,
                1 => 0,
                _ => 1,
            }
        } else {
            u128::from(ordinal)
        })
    }
    #[must_use]
    pub const fn is_struct(self) -> bool {
        matches!(
            self,
            Self::ManuallyDrop
                | Self::AtomicU32
                | Self::SliceIter
                | Self::SliceIterMut
                | Self::Range
        )
    }
    #[must_use]
    pub const fn in_prelude(self) -> bool {
        matches!(self, Self::Option | Self::Result)
    }
    #[must_use]
    pub const fn parameters(self) -> &'static [LibraryParameter] {
        match self {
            Self::Option | Self::Range => &[LibraryParameter::Type {
                name: "T",
                sized: true,
            }],
            Self::ManuallyDrop => &[LibraryParameter::Type {
                name: "T",
                sized: false,
            }],
            Self::Result => &[
                LibraryParameter::Type {
                    name: "T",
                    sized: true,
                },
                LibraryParameter::Type {
                    name: "E",
                    sized: true,
                },
            ],
            Self::SliceIter | Self::SliceIterMut => &[
                LibraryParameter::Lifetime("'a"),
                LibraryParameter::Type {
                    name: "T",
                    sized: true,
                },
            ],
            Self::AtomicU32 | Self::Ordering | Self::ComparisonOrdering => &[],
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
            Self::SliceIter => &[("Iter", Some(LibraryField::SharedSlice))],
            Self::SliceIterMut => &[("IterMut", Some(LibraryField::MutableSlice))],
            Self::Range => &[("Range", Some(LibraryField::RangeBounds))],
            Self::ComparisonOrdering => &[("Less", None), ("Equal", None), ("Greater", None)],
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
