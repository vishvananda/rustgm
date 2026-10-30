//! Source-provider admission consumes retained root attributes before any fallback.
use super::{Context, Source, SourceCore};
use crate::{
    resolve::{
        package::Error,
        stability::Library,
        unit::{FileId, Unit},
    },
    support::bindings::Mode,
    syntax::attribute::Input,
};

#[derive(Default)]
pub(in crate::resolve) struct Features(u64);
#[derive(Clone, Copy)]
pub(in crate::resolve) enum Feature {
    NoCore,
    LangItems,
    Intrinsics,
    RustcAttributes,
    NeverType,
    ExpressionAttributes,
    ConstTrait,
    UnboxedClosures,
    AutoTraits,
    NegativeImpls,
    FreezeImpls,
    Fundamental,
    DeclarativeMacros,
    InternalMacroAllowance,
    InternalUnsafeLint,
    CfgUbChecks,
    F16,
    F128,
    ImplRestriction,
    MinSpecialization,
    Specialization,
    TraitAlias,
    PatternTypeMacro,
    PatternTypes,
    ExternTypes = 24,
    FinalAssociatedFunctions = 25,
    MarkerTraitAttr = 26,
    TryBlocks = 27,
    ConstClosures = 28,
    MetavariableConcat = 29,
    CoverageAttribute = 30,
    MetavariableExpressions = 31,
    CfgTargetHasAtomic = 32,
    ReprSimd = 33,
    SimdFfi = 34,
    CpuApx = 35,
    CpuAvx10 = 36,
    CpuClflushopt = 37,
    CpuErmsb = 38,
    CpuFma4 = 39,
    CpuLahfsahf = 40,
    CpuMovrs = 41,
    CpuPrfchw = 42,
    CpuRtm = 43,
    CpuAmx = 44,
    CpuX87 = 45,
    CpuXop = 46,
    CfgSanitize = 47,
    BuiltinSyntax = 48,
    OffsetOfEnum = 49,
    DeprecatedSuggestion = 50,
    PreludeImport = 51,
    RustdocInternals = 52,
}

impl Feature {
    pub(super) const fn cpu_gate(gate: crate::support::cpu_features::Gate) -> Self {
        use crate::support::cpu_features::Gate;
        match gate {
            Gate::Apx => Self::CpuApx,
            Gate::Avx10 => Self::CpuAvx10,
            Gate::Clflushopt => Self::CpuClflushopt,
            Gate::Ermsb => Self::CpuErmsb,
            Gate::Fma4 => Self::CpuFma4,
            Gate::Lahfsahf => Self::CpuLahfsahf,
            Gate::Movrs => Self::CpuMovrs,
            Gate::Prfchw => Self::CpuPrfchw,
            Gate::Rtm => Self::CpuRtm,
            Gate::Amx => Self::CpuAmx,
            Gate::X87 => Self::CpuX87,
            Gate::Xop => Self::CpuXop,
        }
    }
}
/// Recognizing a gate request is not evidence that its active constructs work.
enum Request {
    Capability(Feature),
    /// A requested library instability still requires its actual declaration inventory.
    Library(Feature),
    /// This target-independent request grants no assembly binding or lowering.
    NoCapability,
}
impl Request {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "prelude_import" => Self::Capability(Feature::PreludeImport),
            "no_core" => Self::Capability(Feature::NoCore),
            "lang_items" => Self::Capability(Feature::LangItems),
            "intrinsics" => Self::Capability(Feature::Intrinsics),
            "rustc_attrs" => Self::Capability(Feature::RustcAttributes),
            // This is a pinned language feature, not a library instability.
            // Documentation metadata grants no type, body or execution authority.
            "rustdoc_internals" => Self::Capability(Feature::RustdocInternals),
            "never_type" => Self::Capability(Feature::NeverType),
            "stmt_expr_attributes" => Self::Capability(Feature::ExpressionAttributes),
            "const_trait_impl" => Self::Capability(Feature::ConstTrait),
            "unboxed_closures" => Self::Capability(Feature::UnboxedClosures),
            "auto_traits" => Self::Capability(Feature::AutoTraits),
            "negative_impls" => Self::Capability(Feature::NegativeImpls),
            "freeze_impls" => Self::Capability(Feature::FreezeImpls),
            "fundamental" => Self::Capability(Feature::Fundamental),
            "decl_macro" => Self::Capability(Feature::DeclarativeMacros),
            "allow_internal_unstable" => Self::Capability(Feature::InternalMacroAllowance),
            "allow_internal_unsafe" => Self::Capability(Feature::InternalUnsafeLint),
            "cfg_ub_checks" => Self::Capability(Feature::CfgUbChecks),
            "cfg_target_has_atomic" => Self::Capability(Feature::CfgTargetHasAtomic),
            "cfg_sanitize" => Self::Capability(Feature::CfgSanitize),
            "builtin_syntax" => Self::Capability(Feature::BuiltinSyntax),
            "offset_of_enum" => Self::Capability(Feature::OffsetOfEnum),
            "deprecated_suggestion" => Self::Capability(Feature::DeprecatedSuggestion),
            "f16" => Self::Capability(Feature::F16),
            "f128" => Self::Capability(Feature::F128),
            "impl_restriction" => Self::Capability(Feature::ImplRestriction),
            "min_specialization" => Self::Capability(Feature::MinSpecialization),
            "specialization" => Self::Capability(Feature::Specialization),
            "trait_alias" => Self::Capability(Feature::TraitAlias),
            "extern_types" => Self::Capability(Feature::ExternTypes),
            "final_associated_functions" => Self::Capability(Feature::FinalAssociatedFunctions),
            "marker_trait_attr" => Self::Capability(Feature::MarkerTraitAttr),
            "try_blocks" => Self::Capability(Feature::TryBlocks),
            "const_closures" => Self::Capability(Feature::ConstClosures),
            "macro_metavar_expr_concat" => Self::Capability(Feature::MetavariableConcat),
            "macro_metavar_expr" => Self::Capability(Feature::MetavariableExpressions),
            "repr_simd" => Self::Capability(Feature::ReprSimd),
            "simd_ffi" => Self::Capability(Feature::SimdFfi),
            "coverage_attribute" => Self::Capability(Feature::CoverageAttribute),
            "pattern_type_macro" => Self::Library(Feature::PatternTypeMacro),
            "pattern_types" => Self::Capability(Feature::PatternTypes),
            // Requests do not authorize the unsupported live const-drop operation.
            "const_destruct" | "const_precise_live_drops" | "asm_experimental_arch" => {
                Self::NoCapability
            }
            // This pinned Rust request only enables an advisory lint. Its
            // producer stays deferred and grants no trait/upcasting permission.
            "multiple_supertrait_upcastable" => Self::NoCapability,
            _ => Self::Capability(Feature::cpu_gate(
                crate::support::cpu_features::Gate::named(name)?,
            )),
        })
    }
}
impl Features {
    pub(in crate::resolve) fn has(&self, feature: Feature) -> bool {
        self.0 & (1_u64 << feature as u8) != 0
    }
    fn needs_local_owner(&self) -> bool {
        self.has(Feature::PreludeImport)
            || self.has(Feature::InternalMacroAllowance)
            || crate::support::cpu_features::Gate::ALL
                .into_iter()
                .any(|gate| self.has(Feature::cpu_gate(gate)))
            || self.has(Feature::ReprSimd)
            || self.has(Feature::SimdFfi)
            || self.has(Feature::CfgTargetHasAtomic)
            || self.has(Feature::CfgSanitize)
            || self.has(Feature::BuiltinSyntax)
            || self.has(Feature::OffsetOfEnum)
            || self.has(Feature::DeprecatedSuggestion)
            || self.has(Feature::ImplRestriction)
            || self.has(Feature::MinSpecialization)
            || self.has(Feature::Specialization)
            || self.has(Feature::TraitAlias)
            || self.has(Feature::PatternTypeMacro)
            || self.has(Feature::PatternTypes)
            || self.has(Feature::ExternTypes)
            || self.has(Feature::FinalAssociatedFunctions)
            || self.has(Feature::MarkerTraitAttr)
            || self.has(Feature::TryBlocks)
            || self.has(Feature::ConstClosures)
            || self.has(Feature::MetavariableConcat)
            || self.has(Feature::MetavariableExpressions)
    }
    fn insert(&mut self, feature: Feature) {
        self.0 |= 1_u64 << feature as u8;
    }
    pub(super) fn macros(&self) -> MacroFeatures {
        MacroFeatures(
            u8::from(self.has(Feature::DeclarativeMacros))
                | (u8::from(self.has(Feature::RustcAttributes)) << 1)
                | (u8::from(self.has(Feature::UnboxedClosures)) << 2)
                | (u8::from(self.has(Feature::CfgUbChecks)) << 5)
                | (u8::from(self.has(Feature::F16)) << 6)
                | (u8::from(self.has(Feature::F128)) << 7),
        )
    }
}

/// Language requests survive ordinary loading without creating a source provider.
/// Internal attributes and trait-parenthesis requests retain distinct checked bits.
#[derive(Clone, Copy, Default)]
pub(in crate::resolve) struct MacroFeatures(u8);
impl MacroFeatures {
    pub(in crate::resolve) fn wide_float(self, half: bool) -> bool {
        self.0 & (1 << if half { 6 } else { 7 }) != 0
    }
    fn with_options(self, options: super::root_options::Options) -> Self {
        Self(self.0 | options.bits())
    }
    pub(in crate::resolve) fn coherence(self) -> bool {
        self.0 & (1 << 3) != 0
    }
    pub(in crate::resolve) fn preserve_ub(self) -> bool {
        self.0 & (1 << 4) != 0
    }
    pub(in crate::resolve) fn cfg_ub(self) -> bool {
        self.0 & (1 << 5) != 0
    }
    pub(in crate::resolve) fn internal(self) -> bool {
        self.0 & 2 != 0
    }
    pub(in crate::resolve) fn unboxed(self) -> bool {
        self.0 & 4 != 0
    }
    pub(in crate::resolve) fn require_scoped(self, file: FileId, offset: u32) -> Result<(), Error> {
        if self.0 & 1 == 0 {
            return Err(Error::at(file, offset, "scoped macros require decl_macro"));
        }
        Ok(())
    }
    pub(in crate::resolve) fn require_internal(
        self,
        file: FileId,
        offset: u32,
    ) -> Result<(), Error> {
        if self.0 & 2 == 0 {
            return Err(Error::at(
                file,
                offset,
                "macro transparency requires rustc_attrs",
            ));
        }
        Ok(())
    }
}

pub(in crate::resolve) struct Prepared {
    pub source: Option<Box<Source>>,
    pub library: Option<Box<Library>>,
    pub macros: MacroFeatures,
}

#[cfg(test)]
pub(in crate::resolve) fn prepare(unit: &Unit, context: &Context) -> Result<Prepared, Error> {
    prepare_with_admission(
        unit,
        context,
        None,
        crate::expand::config::SourceContractLimits::default(),
    )
}

pub(in crate::resolve) fn prepare_with_admission(
    unit: &Unit,
    context: &Context,
    admission: Option<super::consumer_policy::Request<'_>>,
    limits: crate::expand::config::SourceContractLimits,
) -> Result<Prepared, Error> {
    let root = unit.configured.ast.root;
    let no_core = unit
        .word_attribute(Some(root), "no_core")
        .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
    let (features, library) = features(unit)?;
    let options = super::root_options::prepare(unit, features.has(Feature::RustcAttributes))?;
    if let Some(request) = admission {
        let mut prepared =
            super::consumer_policy::prepare(unit, context, no_core, features, library, request)?;
        prepared.macros = prepared.macros.with_options(options);
        return Ok(prepared);
    }
    if let Mode::SourceClient { provider } = &context.mode {
        let mut prepared =
            super::client::prepare(unit, context, provider, no_core, features, library)?;
        prepared.macros = prepared.macros.with_options(options);
        return Ok(prepared);
    }
    let macros = features.macros().with_options(options);
    let Mode::Source { provider } = &context.mode else {
        if no_core {
            return Err(Error::at(
                FileId(0),
                0,
                "no_core requires explicit source-core configuration",
            ));
        }
        return Ok(Prepared {
            source: if features.needs_local_owner() {
                Some(Box::new(Source::LocalMacros(super::LocalMacros::new(
                    unit, features, limits,
                )?)))
            } else {
                None
            },
            library,
            macros,
        });
    };
    let current = context.current.as_ref().ok_or_else(|| {
        Error::at(
            FileId(0),
            0,
            "source-core configuration requires complete current crate identity",
        )
    })?;
    if current != provider || context.crate_name.as_deref() != Some(current.name()) {
        return Err(Error::at(
            FileId(0),
            0,
            "source-core provider differs from current crate identity",
        ));
    }
    if !no_core {
        return Err(Error::at(
            FileId(0),
            0,
            "source-core provider requires no_core",
        ));
    }
    if !features.has(Feature::NoCore) {
        return Err(Error::at(
            FileId(0),
            0,
            "no_core requires its explicit source feature",
        ));
    }
    Ok(Prepared {
        source: Some(Box::new(Source::Core(SourceCore {
            current: current.clone(),
            provider: provider.clone(),
            features,
            macros: super::panic_contracts::Contracts::new(limits),
        }))),
        library,
        macros,
    })
}

fn features(unit: &Unit) -> Result<(Features, Option<Box<Library>>), Error> {
    let mut result = Features::default();
    let mut library: Option<Box<Library>> = None;
    let Some(index) = unit.owner_indices[unit.configured.ast.root.get() as usize] else {
        return Ok((result, library));
    };
    let owner = &unit.configured.owners[index.get() as usize - 1];
    for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
        if attribute.name(forest, unit.source.text()) != Some("feature") {
            continue;
        }
        let failure = || {
            Error::at(
                FileId(0),
                attribute.span.start,
                "invalid source feature request",
            )
        };
        let Input::Delimited(group) = attribute.input else {
            return Err(failure());
        };
        let range = forest.children(group).ok_or_else(failure)?;
        let mut cursor = range.start;
        while cursor < range.end {
            let name = forest
                .name(unit.source.text(), cursor)
                .ok_or_else(failure)?;
            match Request::named(name) {
                Some(Request::Capability(feature)) => result.insert(feature),
                Some(Request::Library(feature)) => {
                    result.insert(feature);
                    library
                        .get_or_insert_with(|| Box::new(Library::default()))
                        .request(name)
                        .map_err(|message| Error::at(FileId(0), attribute.span.start, message))?;
                }
                Some(Request::NoCapability) => {}
                None => {
                    library
                        .get_or_insert_with(|| Box::new(Library::default()))
                        .request(name)
                        .map_err(|message| Error::at(FileId(0), attribute.span.start, message))?;
                }
            }
            cursor += 1;
            if cursor < range.end {
                if !forest.punctuation(cursor, crate::lex::token::Punctuation::Comma) {
                    return Err(failure());
                }
                cursor += 1;
            }
        }
    }
    Ok((result, library))
}

#[cfg(test)]
#[path = "../../../tests/invariants/root_gates.rs"]
mod tests;
