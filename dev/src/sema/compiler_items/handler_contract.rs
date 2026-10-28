//! Actual handler and weak imports share the existing foreign registry transaction.
#[cfg(test)]
#[path = "../../../tests/invariants/handler_execution.rs"]
mod source_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/handler_inputs.rs"]
mod tests;
use super::{Declaration, DefId, Error, LangItem, Report};
use crate::{
    sema::{
        foreign::{Function as ForeignFunction, Registry},
        header_attributes::{self, ContractKnowledge},
        model::{Declarations, Function},
    },
    support::linkage::{Linkage, Naming},
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, ItemKind, Safety},
    },
    types::{
        Kind, RegionId, TypeId, arguments::Argument, callable::Convention, relations::Constraints,
    },
};

type Callable = (DefId, ForeignFunction);

/// Constructed only after current source/header/body and complete callable checks.
pub(in crate::sema) struct Bindings {
    handler: Option<Callable>,
    imports: Vec<Callable>,
}
impl Bindings {
    pub(in crate::sema) fn into_parts(self) -> (Option<Callable>, Vec<Callable>) {
        (self.handler, self.imports)
    }
}

pub(crate) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let declaration = declaration(report, facts)?;
    if report.package.source_core().is_none() {
        return Ok(());
    }
    // Weak imports remain valid object dependencies without a local handler.
    // Both positive source identities are checked before shared naming publishes.
    let imports = imports(report, facts)?;
    if declaration.is_none() && imports.is_empty() {
        return Ok(());
    }
    // Declaration completion owns the nominal producer. This later callable
    // consumer requires its already completed, current source identities.
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("panic callable lost its source catalog"))?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    catalog
        .validated_language(LangItem::PanicLocation)
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("panic callable requires completed actual Location"))?;
    let info = catalog
        .validated_language(LangItem::PanicInfo)
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("panic callable requires completed actual PanicInfo"))?;
    for &(definition, _) in &imports {
        let function = checked_function(report, facts, definition)?;
        signature(facts, function, info, true)?;
    }
    let handler = declaration
        .map(|declaration| checked_handler(report, facts, declaration, info))
        .transpose()?;
    let batch = Bindings { handler, imports };
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())?;
    if let Some(registry) = facts.foreign.as_deref_mut() {
        registry
            .bind_handler(batch, |growth| payments.charge(growth.work, growth.bytes))
            .map_err(Error::unlocated)
    } else {
        payments
            .charge(1, size_of::<Registry>())
            .map_err(Error::unlocated)?;
        let mut registry = Registry::default();
        registry
            .bind_handler(batch, |growth| payments.charge(growth.work, growth.bytes))
            .map_err(Error::unlocated)?;
        facts.foreign = Some(Box::new(registry));
        Ok(())
    }
}

fn checked_handler(
    report: &Report,
    facts: &mut Declarations,
    declaration: Declaration,
    info: Declaration,
) -> Result<(DefId, ForeignFunction), Error> {
    let handler = checked_function(report, facts, declaration.definition)?;
    if declaration.parent.is_some()
        || handler.node != declaration.origin
        || handler.body.is_none()
        || report
            .resolution
            .foreign_owners
            .contains_key(&handler.definition)
        || !facts.typed_function_bodies.contains(&handler.definition)
    {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "panic handler requires its completed actual local body",
        ));
    }
    require_untracked(report, facts, handler.definition)?;
    let safety = source_safety(report, facts, handler)?;
    signature(facts, handler, info, false)?;
    Ok((
        handler.definition,
        ForeignFunction {
            signature: handler.signature,
            safety,
            naming: Naming::Rust,
            linkage: Linkage::Export,
        },
    ))
}

/// Positive reservation/catalog identity is distinct from callable/body proof.
fn declaration(report: &Report, facts: &Declarations) -> Result<Option<Declaration>, Error> {
    let Some(source) = report.package.source_core() else {
        if facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::PanicImpl))
            .is_some()
        {
            return Err(Error::unlocated(
                "imported panic-handler linkage requires transported handler proof",
            ));
        }
        return Ok(None);
    };
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .filter(|reserved| reserved.provider == source.provider)
        .ok_or_else(|| {
            Error::unlocated("source panic handler lacks its actual reservation owner")
        })?;
    let Some(declaration) = reserved.language(LangItem::PanicImpl) else {
        if facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::PanicImpl))
            .is_some()
        {
            return Err(Error::unlocated(
                "source panic catalog has another actual reservation",
            ));
        }
        return Ok(None);
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("source panic handler lacks its completed catalog"))?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    if catalog.language(LangItem::PanicImpl).is_none_or(|actual| {
        actual.definition != declaration.definition
            || actual.origin != declaration.origin
            || actual.parent != declaration.parent
    }) {
        return Err(Error::unlocated(
            "source panic handler changed its actual canonical reservation",
        ));
    }
    Ok(Some(declaration))
}

fn checked_function(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Result<Function, Error> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("panic function lacks its actual source catalog"))?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .copied()
        .ok_or_else(|| Error::unlocated("panic function lacks its completed current header"))?;
    let view = facts.header_attribute_contracts(report, definition)?;
    if function.definition != definition
        || view
            .origin
            .source()
            .is_none_or(|origin| origin.declaration != function.node)
        || matches!(view.must_use, ContractKnowledge::Unknown)
        || matches!(view.inline, ContractKnowledge::Unknown)
        || !matches!(facts.definitions.get(definition.0 as usize).copied().flatten().and_then(|ty| facts.types.kind(ty)),
            Some(Kind::FunctionItem{item,signature,..}) if item.0 == definition.0 && signature == function.signature)
    {
        return Err(Error::unlocated(
            "panic function changed its actual canonical declaration or callable",
        ));
    }
    Ok(function)
}
fn source_safety(
    report: &Report,
    facts: &Declarations,
    function: Function,
) -> Result<Safety, Error> {
    let unit = report.package.unit(function.node.file);
    let Syntax::Item(index) = unit.configured.ast.node(function.node.node).kind else {
        return Err(Error::unlocated("panic handler lost its actual item"));
    };
    let ItemKind::Function {
        flags,
        abi: Abi::Rust,
        body: Some(body),
        ..
    } = unit.configured.ast.items[index as usize].kind
    else {
        return Err(Error::unlocated(
            "panic handler requires its actual Rust function body",
        ));
    };
    if flags.asynchronous
        || function.body
            != Some(crate::resolve::unit::NodeKey {
                file: function.node.file,
                node: body,
            })
        || flags.constant != facts.is_const_function(function.definition)
    {
        return Err(Error::unlocated(
            "panic handler changed its defining function qualifiers",
        ));
    }
    Ok(flags.safety)
}
fn require_untracked(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Result<(), Error> {
    if facts.checked_caller_tracking(report, definition)? {
        return Err(Error::unlocated(
            "tracked panic-handler ABI adaptation is not implemented",
        ));
    }
    Ok(())
}

fn signature(
    facts: &mut Declarations,
    function: Function,
    info: Declaration,
    weak: bool,
) -> Result<(), Error> {
    let (bound_regions, unsafe_) = signature_shape(facts, function.signature, info)?;
    if weak && !unsafe_ {
        return Err(Error::unlocated(
            "safe weak panic declaration contract is not implemented",
        ));
    }
    let count = facts
        .generics
        .parameters
        .get(&function.definition)
        .map_or(0, Vec::len);
    header_attributes::charge_source_retention(facts, count, 0)?;
    if facts
        .generics
        .parameters
        .get(&function.definition)
        .is_some_and(|parameters| {
            parameters
                .iter()
                .any(|parameter| !matches!(parameter.argument, Argument::Lifetime(_)))
        })
    {
        return Err(Error::unlocated(
            "generic panic-handler contract is not implemented",
        ));
    }
    // This exact finite signature has one reference and one nominal lifetime;
    // the existing relation kernel handles alpha-renaming, variance and binders.
    let nodes = usize::try_from(bound_regions)
        .map_err(|_| Error::unlocated("panic signature binder exceeds its arena"))?
        .saturating_add(8);
    let limit = nodes.saturating_mul(64);
    header_attributes::charge_source_retention(
        facts,
        limit,
        nodes
            .saturating_mul(8)
            .saturating_mul(size_of::<Constraints<'_>>()),
    )?;
    let expected = expected_signature(facts, info, unsafe_)?;
    let mut relations = Constraints::new(&facts.types, &facts.lifetimes.variances, 0, limit)
        .map_err(|error| Error::unlocated(error.to_string()))?;
    relations
        .coerce(function.signature, expected, false, 0)
        .map_err(|error| {
            Error::unlocated(format!(
                "panic callable does not satisfy the full PanicInfo lifetime signature: {error}"
            ))
        })?;
    relations.check_aliases().map_err(|error| {
        Error::unlocated(format!(
            "panic callable does not satisfy the full PanicInfo lifetime signature: {error}"
        ))
    })?;
    Ok(())
}

/// Enforce the finite nominal schema before allocating a callable relation graph.
fn signature_shape(
    facts: &Declarations,
    signature: TypeId,
    info: Declaration,
) -> Result<(u32, bool), Error> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(Error::unlocated(
            "panic callable requires its actual Rust signature",
        ));
    };
    let Some([parameter]) = facts.types.list(parameters) else {
        return Err(Error::unlocated(
            "panic callable requires one actual PanicInfo reference",
        ));
    };
    let Some(Kind::Reference {
        referent,
        mutable: false,
        ..
    }) = facts.types.kind(*parameter)
    else {
        return Err(Error::unlocated(
            "panic callable requires its shared PanicInfo reference",
        ));
    };
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(referent) else {
        return Err(Error::unlocated(
            "panic callable requires its actual PanicInfo nominal",
        ));
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or_else(|| Error::unlocated("panic callable lost its actual nominal arguments"))?;
    if item.0 != info.definition.0
        || result != TypeId::NEVER
        || !matches!(arguments.next(), Some(Argument::Lifetime(_)))
        || arguments.next().is_some()
    {
        return Err(Error::unlocated(
            "panic callable has another nominal, result or lifetime argument shape",
        ));
    }
    Ok((bound_regions, unsafe_))
}
fn expected_signature(
    facts: &mut Declarations,
    info: Declaration,
    unsafe_: bool,
) -> Result<TypeId, Error> {
    let lifetime = facts
        .types
        .intern_arguments(&[Argument::Lifetime(
            RegionId::bound(0, 1).map_err(Error::unlocated)?,
        )])
        .map_err(Error::unlocated)?;
    let nominal = facts
        .types
        .intern(Kind::Adt(crate::types::ItemId(info.definition.0), lifetime))
        .map_err(Error::unlocated)?;
    let parameter = facts
        .types
        .intern(Kind::Reference {
            region: RegionId::bound(0, 0).map_err(Error::unlocated)?,
            mutable: false,
            referent: nominal,
        })
        .map_err(Error::unlocated)?;
    let parameters = facts
        .types
        .intern_list(&[parameter])
        .map_err(Error::unlocated)?;
    facts
        .types
        .intern(Kind::Function {
            bound_regions: 2,
            parameters,
            result: TypeId::NEVER,
            unsafe_,
            convention: Convention::RUST,
        })
        .map_err(Error::unlocated)
}

fn plain_import_name(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    record: ForeignFunction,
) -> Result<(), Error> {
    let source = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(|| Error::unlocated("weak panic source definition disappeared"))?;
    let spelling = report.package.names.get(source.name);
    header_attributes::charge_source_retention(facts, spelling.len().saturating_add(18), 0)?;
    let actual = facts
        .foreign
        .as_deref()
        .and_then(|registry| match record.naming {
            crate::support::linkage::Naming::Exact(name) => registry.names().get(name),
            crate::support::linkage::Naming::Rust => None,
        });
    if !actual.is_some_and(|actual| actual == spelling || actual == "rust_begin_unwind") {
        return Err(Error::unlocated(
            "weak panic import changed its actual default source symbol",
        ));
    }
    Ok(())
}

fn imports(
    report: &Report,
    facts: &mut Declarations,
) -> Result<Vec<(DefId, ForeignFunction)>, Error> {
    let count = report.resolution.foreign_owners.len();
    let sorting = count.saturating_mul(
        usize::try_from(usize::BITS - count.max(1).leading_zeros()).unwrap_or(usize::MAX),
    );
    header_attributes::charge_source_retention(
        facts,
        count.saturating_add(sorting).saturating_add(1),
        count.saturating_mul(size_of::<(DefId, ForeignFunction)>()),
    )?;
    let mut imports = Vec::with_capacity(count);
    for &definition in report.resolution.foreign_owners.keys() {
        if report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .is_some_and(|row| row.kind == crate::resolve::model::DefKind::Function)
        {
            let record = facts
                .foreign
                .as_deref()
                .and_then(|registry| registry.function(definition))
                .ok_or_else(|| {
                    Error::unlocated("foreign callable lacks its actual registry record")
                })?;
            imports.push((definition, record));
        }
    }
    imports.sort_unstable_by_key(|row| row.0.0);
    let mut retained = 0;
    for index in 0..imports.len() {
        let (definition, record) = imports[index];
        if !facts.weak_panic_import(report, definition)? {
            continue;
        }
        let function = checked_function(report, facts, definition)?;
        require_untracked(report, facts, definition)?;
        plain_import_name(report, facts, definition, record)?;
        if function.body.is_some()
            || record.signature != function.signature
            || record.linkage != Linkage::Import
        {
            return Err(Error::unlocated(
                "weak panic import changed its actual bodyless callable record",
            ));
        }
        imports[retained] = imports[index];
        retained += 1;
    }
    imports.truncate(retained);
    Ok(imports)
}
