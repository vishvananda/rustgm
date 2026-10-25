//! Pinned compiler role identity, invocation kind and execution availability.
//! Source: rust88d9 compiler/rustc_builtin_macros/src/lib.rs; declarations alone grant no execution.
/// Invocation category belongs to the compiler role, not its declaration syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinKind {
    Bang,
    Attribute,
    Derive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Cfg,
    CfgSelect,
    FormatArgs,
    ConstFormatArgs,
    FormatArgsNl,
    Line,
    Column,
    File,
    ModulePath,
    Stringify,
    Concat,
    CompileError,
    Env,
    IncludeStr,
    IncludeBytes,
    Include,
    OptionEnv,
    CorePanic,
    StdPanic,
    Unreachable,
    ConcatBytes,
    AutodiffForward,
    AutodiffReverse,
    Test,
    Asm,
    Assert,
    GlobalAsm,
    Iter,
    LogSyntax,
    NakedAsm,
    PatternType,
    TraceMacros,
    AllocErrorHandler,
    Bench,
    CfgAccessible,
    CfgEval,
    DefineOpaque,
    Derive,
    DeriveConst,
    Eii,
    EiiDeclaration,
    EiiSharedMacro,
    GlobalAllocator,
    OffloadKernel,
    TestCase,
    UnsafeEii,
    Clone,
    Copy,
    ConstParamTy,
    Debug,
    Default,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    CoercePointee,
    From,
    Quote,
    ContractsRequires,
    ContractsEnsures,
}
impl Builtin {
    /// Stable original prefix: ordinary crates allocate only these synthetic slots.
    pub const BOOTSTRAP: [Self; 24] = [
        Self::Cfg,
        Self::CfgSelect,
        Self::FormatArgs,
        Self::ConstFormatArgs,
        Self::FormatArgsNl,
        Self::Line,
        Self::Column,
        Self::File,
        Self::ModulePath,
        Self::Stringify,
        Self::Concat,
        Self::CompileError,
        Self::Env,
        Self::IncludeStr,
        Self::IncludeBytes,
        Self::Include,
        Self::OptionEnv,
        Self::CorePanic,
        Self::StdPanic,
        Self::Unreachable,
        Self::ConcatBytes,
        Self::AutodiffForward,
        Self::AutodiffReverse,
        Self::Test,
    ];
    /// Finite identities from the pinned compiler registry, independent of execution.
    pub const ALL: [Self; 61] = [
        Self::Cfg,
        Self::CfgSelect,
        Self::FormatArgs,
        Self::ConstFormatArgs,
        Self::FormatArgsNl,
        Self::Line,
        Self::Column,
        Self::File,
        Self::ModulePath,
        Self::Stringify,
        Self::Concat,
        Self::CompileError,
        Self::Env,
        Self::IncludeStr,
        Self::IncludeBytes,
        Self::Include,
        Self::OptionEnv,
        Self::CorePanic,
        Self::StdPanic,
        Self::Unreachable,
        Self::ConcatBytes,
        Self::AutodiffForward,
        Self::AutodiffReverse,
        Self::Test,
        Self::Asm,
        Self::Assert,
        Self::GlobalAsm,
        Self::Iter,
        Self::LogSyntax,
        Self::NakedAsm,
        Self::PatternType,
        Self::TraceMacros,
        Self::AllocErrorHandler,
        Self::Bench,
        Self::CfgAccessible,
        Self::CfgEval,
        Self::DefineOpaque,
        Self::Derive,
        Self::DeriveConst,
        Self::Eii,
        Self::EiiDeclaration,
        Self::EiiSharedMacro,
        Self::GlobalAllocator,
        Self::OffloadKernel,
        Self::TestCase,
        Self::UnsafeEii,
        Self::Clone,
        Self::Copy,
        Self::ConstParamTy,
        Self::Debug,
        Self::Default,
        Self::Eq,
        Self::Hash,
        Self::Ord,
        Self::PartialEq,
        Self::PartialOrd,
        Self::CoercePointee,
        Self::From,
        Self::Quote,
        Self::ContractsRequires,
        Self::ContractsEnsures,
    ];
    pub(crate) const fn bootstrap_slot(self) -> Option<usize> {
        let slot = self as usize;
        if slot < Self::BOOTSTRAP.len() {
            Some(slot)
        } else {
            None
        }
    }
    pub(crate) const fn unavailable_message(self) -> &'static str {
        match self.kind() {
            BuiltinKind::Bang => "builtin bang macro execution is not implemented",
            BuiltinKind::Attribute => "builtin attribute execution is not implemented",
            BuiltinKind::Derive => "builtin derive macro execution is not implemented",
        }
    }
    pub(crate) const fn frame_eligible(self) -> bool {
        matches!(self.kind(), BuiltinKind::Bang) && self.executable()
    }
    #[must_use]
    pub const fn kind(self) -> BuiltinKind {
        match self {
            Self::AllocErrorHandler
            | Self::AutodiffForward
            | Self::AutodiffReverse
            | Self::Bench
            | Self::CfgAccessible
            | Self::CfgEval
            | Self::DefineOpaque
            | Self::Derive
            | Self::DeriveConst
            | Self::Eii
            | Self::EiiDeclaration
            | Self::EiiSharedMacro
            | Self::GlobalAllocator
            | Self::OffloadKernel
            | Self::Test
            | Self::TestCase
            | Self::UnsafeEii
            | Self::ContractsRequires
            | Self::ContractsEnsures => BuiltinKind::Attribute,
            Self::Clone
            | Self::Copy
            | Self::ConstParamTy
            | Self::Debug
            | Self::Default
            | Self::Eq
            | Self::Hash
            | Self::Ord
            | Self::PartialEq
            | Self::PartialOrd
            | Self::CoercePointee
            | Self::From => BuiltinKind::Derive,
            _ => BuiltinKind::Bang,
        }
    }
    /// Only roles with a real expansion or owner-removal consumer are available.
    #[must_use]
    pub const fn executable(self) -> bool {
        matches!(
            self,
            Self::Cfg
                | Self::CfgSelect
                | Self::FormatArgs
                | Self::ConstFormatArgs
                | Self::FormatArgsNl
                | Self::Line
                | Self::Column
                | Self::File
                | Self::ModulePath
                | Self::Stringify
                | Self::Concat
                | Self::CompileError
                | Self::Env
                | Self::IncludeStr
                | Self::IncludeBytes
                | Self::Include
                | Self::OptionEnv
                | Self::CorePanic
                | Self::StdPanic
                | Self::Unreachable
                | Self::ConcatBytes
                | Self::Test
        )
    }
    #[must_use]
    pub const fn source_only(self) -> bool {
        self.bootstrap_slot().is_none()
            || matches!(
                self,
                Self::CorePanic
                    | Self::StdPanic
                    | Self::Unreachable
                    | Self::CfgSelect
                    | Self::FormatArgs
                    | Self::ConstFormatArgs
                    | Self::FormatArgsNl
                    | Self::ConcatBytes
                    | Self::AutodiffForward
                    | Self::AutodiffReverse
                    | Self::Test
            )
    }
    #[must_use]
    pub const fn formatting(self) -> bool {
        matches!(
            self,
            Self::FormatArgs | Self::ConstFormatArgs | Self::FormatArgsNl
        )
    }
    #[must_use]
    pub const fn source_eager(self) -> bool {
        self.formatting() || matches!(self, Self::ConcatBytes)
    }
    #[must_use]
    pub const fn eager(self) -> bool {
        matches!(
            self,
            Self::ConcatBytes
                | Self::FormatArgs
                | Self::ConstFormatArgs
                | Self::FormatArgsNl
                | Self::OptionEnv
                | Self::Concat
                | Self::CompileError
                | Self::Env
                | Self::IncludeStr
                | Self::IncludeBytes
                | Self::Include
        )
    }
    #[must_use]
    pub const fn needs_position(self) -> bool {
        matches!(self, Self::Line | Self::Column)
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cfg => "cfg",
            Self::CfgSelect => "cfg_select",
            Self::FormatArgs => "format_args",
            Self::ConstFormatArgs => "const_format_args",
            Self::FormatArgsNl => "format_args_nl",
            Self::Line => "line",
            Self::Column => "column",
            Self::File => "file",
            Self::ModulePath => "module_path",
            Self::Stringify => "stringify",
            Self::Concat => "concat",
            Self::ConcatBytes => "concat_bytes",
            Self::AutodiffForward => "autodiff_forward",
            Self::AutodiffReverse => "autodiff_reverse",
            Self::Test => "test",
            Self::CompileError => "compile_error",
            Self::Env => "env",
            Self::OptionEnv => "option_env",
            Self::IncludeStr => "include_str",
            Self::IncludeBytes => "include_bytes",
            Self::Include => "include",
            Self::CorePanic => "core_panic",
            Self::StdPanic => "std_panic",
            Self::Unreachable => "unreachable",
            Self::Asm => "asm",
            Self::Assert => "assert",
            Self::GlobalAsm => "global_asm",
            Self::Iter => "iter",
            Self::LogSyntax => "log_syntax",
            Self::NakedAsm => "naked_asm",
            Self::PatternType => "pattern_type",
            Self::TraceMacros => "trace_macros",
            Self::AllocErrorHandler => "alloc_error_handler",
            Self::Bench => "bench",
            Self::CfgAccessible => "cfg_accessible",
            Self::CfgEval => "cfg_eval",
            Self::DefineOpaque => "define_opaque",
            Self::Derive => "derive",
            Self::DeriveConst => "derive_const",
            Self::Eii => "eii",
            Self::EiiDeclaration => "eii_declaration",
            Self::EiiSharedMacro => "eii_shared_macro",
            Self::GlobalAllocator => "global_allocator",
            Self::OffloadKernel => "offload_kernel",
            Self::TestCase => "test_case",
            Self::UnsafeEii => "unsafe_eii",
            Self::Clone => "Clone",
            Self::Copy => "Copy",
            Self::ConstParamTy => "ConstParamTy",
            Self::Debug => "Debug",
            Self::Default => "Default",
            Self::Eq => "Eq",
            Self::Hash => "Hash",
            Self::Ord => "Ord",
            Self::PartialEq => "PartialEq",
            Self::PartialOrd => "PartialOrd",
            Self::CoercePointee => "CoercePointee",
            Self::From => "From",
            Self::Quote => "quote",
            Self::ContractsRequires => "contracts_requires",
            Self::ContractsEnsures => "contracts_ensures",
        }
    }
}
