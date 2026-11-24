//! Checked imported intrinsic declarations carry no call or evaluation proof.
use super::{CallerLocationSelection, Catalog, LangItem, inhabited_properties::Plans};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    support::{
        bindings::SourceIntrinsic, crates::generics::source::BodylessIntrinsic, hash::HashMap,
    },
    types::{Kind, Primitive, TypeId, arguments::Argument},
};

#[derive(Clone, Copy)]
pub(super) struct Header {
    pub(super) definition: DefId,
    pub(super) signature: TypeId,
    result: TypeId,
}
#[derive(Clone, Copy)]
pub(super) struct Declared {
    pub(super) origin: crate::resolve::body_origin::DeclarationOrigin,
    pub(super) kind: SourceIntrinsic,
    pub(super) signature: TypeId,
    pub(super) source_fallback: bool,
}
pub(super) type Headers = [Option<Header>; BodylessIntrinsic::ALL.len()];
pub(super) type DeclaredIntrinsics = HashMap<DefId, Declared>;

pub(super) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let mut steps = 0;
    let checked = check(report, facts, &mut steps);
    let declared = match &checked {
        Ok(_) => check_declared(facts, &mut steps),
        Err(message) => Err(*message),
    };
    let has_bodyless = checked.as_ref().is_ok_and(Option::is_some);
    let declared_count = declared
        .as_ref()
        .ok()
        .and_then(Option::as_ref)
        .map_or(0, HashMap::len);
    let plans = facts
        .source_bindings
        .as_deref()
        .is_some_and(|catalog| catalog.native_layouts.is_none());
    let storage = usize::from(has_bodyless) * size_of::<Headers>()
        + usize::from(declared_count > 0)
            * (size_of::<DeclaredIntrinsics>()
                + crate::support::retained::new_map::<DefId, Declared>(declared_count))
        + usize::from(plans && (has_bodyless || declared_count > 0)) * size_of::<Plans>();
    crate::sema::crates::charge_source_headers(facts, steps, storage)?;
    let headers = checked.map_err(Error::unlocated)?;
    let declared = declared.map_err(Error::unlocated)?;
    if headers.is_some() || declared.is_some() {
        let plans = facts
            .source_bindings
            .as_deref_mut()
            .ok_or_else(|| Error::unlocated("imported intrinsic lost its catalog"))?
            .native_layouts
            .get_or_insert_with(Default::default);
        if let Some(headers) = headers {
            plans.imported_bodyless = Some(Box::new(headers));
        }
        if let Some(declared) = declared {
            plans.imported_declared = Some(Box::new(declared));
        }
    }
    Ok(())
}

fn check_declared(
    facts: &Declarations,
    steps: &mut usize,
) -> Result<Option<DeclaredIntrinsics>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported intrinsic lost its catalog")?;
    let entries = catalog.declared_intrinsics(steps)?;
    if entries.is_empty() {
        return Ok(None);
    }
    let mut validated = DeclaredIntrinsics::default();
    validated
        .try_reserve(entries.len())
        .map_err(|_| "imported intrinsic table allocation failed")?;
    for (role, kind, source_fallback) in entries {
        *steps = steps.saturating_add(2);
        let actual = facts
            .declaration_header(role.definition)?
            .ok_or("imported declared intrinsic lost its canonical declaration")?;
        if actual.declaration != role.origin
            || (source_fallback
                && (actual.body.is_none() || !facts.is_const_function(role.definition)))
        {
            return Err(
                "imported declared intrinsic changed its declaration origin, body or effect",
            );
        }
        if validated
            .insert(
                role.definition,
                Declared {
                    origin: role.origin,
                    kind,
                    signature: actual.signature,
                    source_fallback,
                },
            )
            .is_some()
        {
            return Err("imported declared intrinsic is duplicated");
        }
    }
    Ok(Some(validated))
}

fn check(
    report: &Report,
    facts: &Declarations,
    steps: &mut usize,
) -> Result<Option<Headers>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported intrinsic lost its catalog")?;
    catalog.require_provider_report(report)?;
    let roles = catalog.bodyless_headers(steps)?;
    let mut validated = [None; BodylessIntrinsic::ALL.len()];
    for (operation, role) in BodylessIntrinsic::ALL.into_iter().zip(roles) {
        let Some(role) = role else { continue };
        let actual = facts
            .declaration_header(role.definition)?
            .ok_or("imported intrinsic lost its canonical declaration")?;
        if actual.declaration != role.origin
            || actual.universals != 0
            || actual.body.is_some()
            || !facts.is_const_function(role.definition)
        {
            return Err(
                "imported intrinsic changed its declaration origin, effect or signature scope",
            );
        }
        let result = match operation {
            BodylessIntrinsic::CallerLocation => {
                catalog
                    .validated_role(LangItem::PanicLocation)?
                    .ok_or("imported caller_location requires its checked Location nominal")?;
                super::caller_location::validate_signature(
                    facts,
                    role.definition,
                    actual.signature,
                    false,
                )?
            }
            BodylessIntrinsic::PtrMetadata => {
                super::ptr_metadata::validate_header(facts, role.definition, actual.signature)?;
                let Some(Kind::Function { result, .. }) = facts.types.kind(actual.signature) else {
                    return Err("imported ptr_metadata lost its checked signature");
                };
                *steps = steps.saturating_add(16).saturating_add(
                    crate::sema::declaration_inputs::declared_own(
                        &facts.generics,
                        facts.crates.as_deref(),
                        role.definition,
                    )?
                    .len(),
                );
                result
            }
            BodylessIntrinsic::OffsetOf => super::displacement::validate_imported(
                facts,
                catalog,
                role,
                actual.signature,
                steps,
            )?,
            BodylessIntrinsic::FieldOffset => super::representing_offsets::validate_imported(
                facts,
                catalog,
                role,
                actual.signature,
                steps,
            )?,
            BodylessIntrinsic::SizeOf | BodylessIntrinsic::AlignOf => {
                validate_layout(facts, role.definition, actual.signature, operation)?;
                TypeId::primitive(Primitive::Usize)
            }
        };
        validated[operation as usize] = Some(Header {
            definition: role.definition,
            signature: actual.signature,
            result,
        });
    }
    Ok(validated.iter().any(Option::is_some).then_some(validated))
}

fn validate_layout(
    facts: &Declarations,
    definition: DefId,
    signature: TypeId,
    operation: BodylessIntrinsic,
) -> Result<(), &'static str> {
    super::intrinsic_contracts::scalar_signature(facts, definition, signature, operation.kind())?;
    let Some([parameter]) = facts
        .generics
        .parameters
        .get(&definition)
        .map(Vec::as_slice)
    else {
        return Err("imported layout intrinsic requires one actual type formal");
    };
    if parameter.default.is_some()
        || parameter.anonymous
        || !matches!(parameter.argument,
        Argument::Type(ty) if matches!(facts.types.kind(ty), Some(Kind::Parameter(id)) if facts.types.parameter(id).is_some_and(|parameter| parameter.index == 0)))
    {
        return Err(
            "imported layout intrinsic requires its own first type formal without a default",
        );
    }
    Ok(())
}

impl Catalog {
    pub(super) fn imported_intrinsic(
        &self,
        definition: DefId,
    ) -> Option<(BodylessIntrinsic, Header)> {
        let headers = self
            .native_layouts
            .as_deref()?
            .imported_bodyless
            .as_deref()?;
        BodylessIntrinsic::ALL
            .into_iter()
            .zip(headers)
            .find_map(|(kind, header)| {
                header
                    .filter(|header| header.definition == definition)
                    .map(|header| (kind, header))
            })
    }

    pub(super) fn imported_declared_intrinsic(&self, definition: DefId) -> Option<Declared> {
        self.native_layouts
            .as_deref()?
            .imported_declared
            .as_deref()?
            .get(&definition)
            .copied()
    }

    pub(super) fn imported_caller_location(
        &self,
        definition: DefId,
    ) -> Option<CallerLocationSelection> {
        let (BodylessIntrinsic::CallerLocation, header) = self.imported_intrinsic(definition)?
        else {
            return None;
        };
        Some(CallerLocationSelection {
            definition,
            signature: header.signature,
            result: header.result,
        })
    }
}
