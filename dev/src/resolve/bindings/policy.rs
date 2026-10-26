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
pub(in crate::resolve) struct Features(u32);
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
            "no_core" => Self::Capability(Feature::NoCore),
            "lang_items" => Self::Capability(Feature::LangItems),
            "intrinsics" => Self::Capability(Feature::Intrinsics),
            "rustc_attrs" => Self::Capability(Feature::RustcAttributes),
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
            "f16" => Self::Capability(Feature::F16),
            "f128" => Self::Capability(Feature::F128),
            "impl_restriction" => Self::Capability(Feature::ImplRestriction),
            "min_specialization" => Self::Capability(Feature::MinSpecialization),
            "specialization" => Self::Capability(Feature::Specialization),
            "trait_alias" => Self::Capability(Feature::TraitAlias),
            "pattern_type_macro" => Self::Library(Feature::PatternTypeMacro),
            "pattern_types" => Self::Capability(Feature::PatternTypes),
            // Requests do not authorize the unsupported live const-drop operation.
            "const_destruct" | "asm_experimental_arch" => Self::NoCapability,
            _ => return None,
        })
    }
}
impl Features {
    pub(in crate::resolve) fn has(&self, feature: Feature) -> bool {
        self.0 & (1_u32 << feature as u8) != 0
    }
    fn insert(&mut self, feature: Feature) {
        self.0 |= 1_u32 << feature as u8;
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

pub(in crate::resolve) fn prepare(unit: &Unit, context: &Context) -> Result<Prepared, Error> {
    prepare_with_admission(unit, context, None)
}

pub(in crate::resolve) fn prepare_with_admission(
    unit: &Unit,
    context: &Context,
    admission: Option<super::consumer_policy::Request<'_>>,
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
            source: if features.has(Feature::InternalMacroAllowance)
                || features.has(Feature::ImplRestriction)
                || features.has(Feature::MinSpecialization)
                || features.has(Feature::Specialization)
                || features.has(Feature::TraitAlias)
                || features.has(Feature::PatternTypeMacro)
                || features.has(Feature::PatternTypes)
            {
                Some(Box::new(Source::LocalMacros(super::LocalMacros::new(
                    unit, features,
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
            macros: super::panic_contracts::Contracts::default(),
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
